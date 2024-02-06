use proc_macro2::TokenTree;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::ToTokens;
use std::collections::HashMap;
use std::str::FromStr;
use syn::parse::ParseStream;
use syn::{parse_macro_input, Attribute, Token};

#[proc_macro_derive(SceneBinding)]
pub fn scene_binding_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let ast: syn::DeriveInput = syn::parse2(input).unwrap();
    let name = &ast.ident;
    let found_crate = crate_name("foliage").expect("foliage is present in `Cargo.toml`");
    let foliage = match found_crate {
        FoundCrate::Itself => quote::quote!(crate),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote::quote!( #ident )
        }
    };
    let gen = match &ast.data {
        syn::Data::Enum(_) => {
            quote::quote! {
                impl From<#name> for #foliage::scene::SceneBinding {
                    fn from(value: #name) -> Self {
                        Self(value as i32)
                    }
                }
            }
        }
        _ => {
            quote::quote! {}
        }
    };
    gen.into()
}
#[proc_macro_derive(InnerSceneBinding)]
pub fn inner_scene_binding_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let ast: syn::DeriveInput = syn::parse2(input).unwrap();
    let name = &ast.ident;
    let found_crate =
        crate_name("foliage_proper").expect("foliage_proper is present in `Cargo.toml`");
    let foliage = match found_crate {
        FoundCrate::Itself => quote::quote!(crate),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote::quote!( #ident )
        }
    };
    let gen = match &ast.data {
        syn::Data::Enum(_) => {
            quote::quote! {
                impl From<#name> for #foliage::scene::SceneBinding {
                    fn from(value: #name) -> Self {
                        Self(value as i32)
                    }
                }
            }
        }
        _ => {
            quote::quote! {}
        }
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn assets(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let asset_configuration = parse_macro_input!(attrs as AssetConfiguration);
    let struct_definition = parse_macro_input!(input as syn::ItemStruct);
    let found_crate = crate_name("foliage").expect("foliage is present in `Cargo.toml`");
    let foliage = match found_crate {
        FoundCrate::Itself => quote::quote!(crate),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote::quote!( #ident )
        }
    };
    let mut gen_fields = HashMap::new();
    for field in struct_definition.fields.iter() {
        let field_attr = field.attrs.first().expect("no attribute on field");
        let asset_type = field_attr
            .path()
            .require_ident()
            .expect("must name attribute [bytes(...) | icon(...)]");
        let args = field_attr
            .parse_args::<Asset>()
            .expect("error parsing asset from attribute");
        let storage_identifier = if args.asset_group.is_some() {
            args.asset_group
                .unwrap()
                .value
                .to_token_stream()
                .to_string()
        } else {
            field
                .ident
                .as_ref()
                .expect("need field identity without group")
                .to_string()
        };
        let icon_extension = if asset_type.to_string() != "icon" {
            quote::quote!()
        } else {
            let icon_label = &args
                .asset_opt
                .expect("must provide opt=FeatherIcon::<variant>")
                .value;
            quote::quote!(
                    elm.on_fetch(
                        id,
                        |data, cmd| {
                            cmd.spawn(
                                #foliage::icon::Icon::storage(
                                    #foliage::icon::#icon_label.id(),
                                    data
                                )
                            );
                        },
                    );
            )
        };
        let engen = &asset_configuration.engen_path;
        let asset_path = &args.asset_path.value;
        let (native, remote) = {
            let native = syn::LitStr::new(
                (asset_configuration.native_path.value()
                    + asset_path.to_token_stream().to_string().as_str())
                .replace(['\"', '\\'], "")
                .as_str(),
                proc_macro2::Span::call_site(),
            );
            let remote = syn::LitStr::new(
                (asset_configuration.web_path.value()
                    + asset_path.to_token_stream().to_string().as_str())
                .replace(['\"', '\\'], "")
                .as_str(),
                proc_macro2::Span::call_site(),
            );
            (native, remote)
        };
        let generated_code = quote::quote!({
            #[cfg(target_family = "wasm")]
            use #foliage::workflow::Workflow;
            #[cfg(target_family = "wasm")]
            let id = elm.load_remote_asset::<#engen, _>(#remote);
            #[cfg(not(target_family = "wasm"))]
            let id = elm.generate_asset_key();
            #[cfg(not(target_family = "wasm"))]
            elm.store_local_asset(id, include_bytes!(#native).to_vec());
            #icon_extension
            id
        });
        if gen_fields.get(&storage_identifier).is_none() {
            gen_fields.insert(storage_identifier.clone(), vec![]);
        }
        gen_fields
            .get_mut(&storage_identifier)
            .unwrap()
            .push(generated_code);
    }
    println!("gen-fields: {:?}", gen_fields);
    let mut field_iterator = vec![];
    let mut loaders = vec![];
    let mut tys = vec![];
    for (id, mut tokens) in gen_fields {
        let real_id = syn::Ident::new(id.as_str(), proc_macro2::Span::call_site());
        field_iterator.push(quote::quote!(#real_id));
        if tokens.len() == 1 {
            let ty = proc_macro2::TokenStream::from_str("AssetKey").expect("asset-key");
            tys.push(quote::quote!(#ty));
            let l = tokens.first().unwrap();
            loaders.push(quote::quote!(#l));
        } else {
            let ty = proc_macro2::TokenStream::from_str("Vec<AssetKey>").expect("vec-parsing");
            tys.push(quote::quote!(#ty));
            let group_members = tokens
                .drain(..)
                .map(|gm| syn::parse2::<syn::Expr>(gm).expect("syn-expr"))
                .collect::<Vec<syn::Expr>>();
            loaders.push(quote::quote!(vec![#(#group_members),*]));
        }
    }
    let forwarded_attrs = struct_definition.attrs;
    let vis = struct_definition.vis;
    let name = struct_definition.ident;
    let gen = quote::quote! {
        #(#forwarded_attrs)*
        #vis struct #name {
            #(#vis #field_iterator: #tys),*
        }
        impl #name {
            #vis fn proc_gen_load(elm: &mut #foliage::elm::Elm) -> Self {
                Self {
                    #(#field_iterator: #loaders),*
                }
            }
        }
    };
    gen.into()
}
struct AssetConfiguration {
    engen_path: syn::TypePath,
    native_path: syn::LitStr,
    web_path: syn::LitStr,
}
impl syn::parse::Parse for AssetConfiguration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let engen_path: syn::TypePath = input.parse()?;
        input.parse::<Token![,]>()?;
        let native_path: syn::LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let web_path: syn::LitStr = input.parse()?;
        Ok(AssetConfiguration {
            engen_path,
            native_path,
            web_path,
        })
    }
}
struct Asset {
    asset_path: syn::MetaNameValue,
    asset_group: Option<syn::MetaNameValue>,
    asset_opt: Option<syn::MetaNameValue>,
}
impl syn::parse::Parse for Asset {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let asset_path: syn::MetaNameValue = input.parse()?;
        let mut asset_group = None;
        let mut asset_opt = None;
        if input.parse::<Token![,]>().is_ok() {
            let second: syn::MetaNameValue = input.parse()?;
            let second_attribute_path = second
                .path
                .require_ident()
                .expect("need attribute args ident");
            if second_attribute_path.to_string() == "group" {
                asset_group.replace(second);
                if input.parse::<Token![,]>().is_ok() {
                    let opt: syn::MetaNameValue = input.parse()?;
                    asset_opt.replace(opt);
                }
            } else if second_attribute_path.to_string() == "opt" {
                asset_opt.replace(second);
                if input.parse::<Token![,]>().is_ok() {
                    let group: syn::MetaNameValue = input.parse()?;
                    asset_group.replace(group);
                }
            } else {
                panic!("unsupported name-value pair");
            }
        }
        Ok(Asset {
            asset_path,
            asset_group,
            asset_opt,
        })
    }
}
