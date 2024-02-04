use proc_macro2::TokenTree;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::ToTokens;
use std::collections::HashMap;
use std::str::FromStr;
use syn::Attribute;

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
#[proc_macro_attribute]
pub fn assets(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    // parse attr to get #engen / #native_origin / #remote_origin
    let mut engen = None;
    let mut native_origin = None;
    let mut remote_origin = None;
    for a in attr {
        match &a {
            TokenTree::Ident(ident) => {
                let id =
                    syn::Ident::new(ident.to_string().as_str(), proc_macro2::Span::call_site());
                engen.replace(id);
            }
            TokenTree::Literal(lit) => {
                if native_origin.is_none() {
                    let lit_str =
                        syn::LitStr::new(lit.to_string().as_str(), proc_macro2::Span::call_site());
                    native_origin.replace(lit_str);
                } else {
                    let lit_str =
                        syn::LitStr::new(lit.to_string().as_str(), proc_macro2::Span::call_site());
                    remote_origin.replace(lit_str);
                }
            }
            _ => {}
        }
    }
    let engen = engen.unwrap();
    let native_origin = native_origin.unwrap();
    let remote_origin = remote_origin.unwrap();
    // end parse
    let modified_input = proc_macro2::TokenStream::from(input);
    let definition: syn::ItemStruct =
        syn::parse2(modified_input).expect("foliage::asset requires a struct definition");
    let name = &definition.ident;
    let mut groups = HashMap::new();
    let found_crate = crate_name("foliage").expect("foliage is present in `Cargo.toml`");
    let foliage = match found_crate {
        FoundCrate::Itself => quote::quote!(crate),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote::quote!( #ident )
        }
    };
    for field in definition.fields.iter() {
        let asset_attributes = field.attrs.clone();
        let is_grouped = if asset_attributes.len() == 2 {
            true
        } else if asset_attributes.len() == 1 {
            false
        } else {
            panic!("need an attribute to know where to load the asset.\nTry adding #[asset(path = \"<path>\" ...");
        };
        let (group_name, asset_type, asset_value) = if is_grouped {
            let first = asset_attributes.first().expect("first-attribute-unwrap");
            let first_identity = first.path().get_ident();
            let first_attribute = first_identity
                .expect("incorrect attribute specification")
                .to_string();
            let second = asset_attributes.get(1).expect("second-attribute");
            let second_identity = second.path().get_ident();
            let second_attribute = second_identity
                .expect("incorrect attribute specification")
                .to_string();
            if first_attribute.contains("group") {
                let group_name = get_group_value(&asset_attributes, 0);
                let asset_type = second_identity.expect("second-identity").to_string();
                let asset_value = second
                    .meta
                    .require_list()
                    .expect("meta-list-parse")
                    .to_token_stream()
                    .to_string();
                (group_name, asset_type, asset_value)
            } else if second_attribute.contains("group") {
                let group_name = get_group_value(&asset_attributes, 1);
                let asset_type = first_identity.expect("asset-type").to_string();
                let asset_value = first
                    .meta
                    .require_list()
                    .expect("meta-list-parse")
                    .to_token_stream()
                    .to_string();
                (group_name, asset_type, asset_value)
            } else {
                panic!("only two attributes allowed && if more than one, one must be a #[group]");
            }
        } else {
            let only = asset_attributes.get(0).expect("only-get");
            let name = only.path().get_ident().expect("only-name").to_string();
            let value = only
                .meta
                .require_list()
                .expect("meta-list-parse")
                .to_token_stream()
                .to_string();
            (
                field.ident.as_ref().expect("field-ident").to_string(),
                name,
                value,
            )
        };
        if groups.get(&group_name).is_none() {
            if is_grouped {
                groups.insert(group_name.clone(), Group::new(AssetStorageType::Vec));
            } else {
                groups.insert(group_name.clone(), Group::new(AssetStorageType::Single));
            }
        }
        let is_icon = asset_type.contains("icon");
        let (res_path, icon_label) = if is_icon {
            let icon_attributes = syn::parse2::<syn::MetaList>(
                proc_macro2::TokenStream::from_str(asset_value.as_str())
                    .expect("icon-attributes-from-asset-value"),
            )
            .expect("no meta list for icon attribute definition");
            let mut icon_path = None;
            let mut icon_label = None;
            for t in icon_attributes.tokens.clone() {
                match t {
                    TokenTree::Group(_) => {}
                    TokenTree::Ident(i) => {
                        icon_label.replace(syn::Ident::new(
                            i.to_string().as_str(),
                            proc_macro2::Span::call_site(),
                        ));
                    }
                    TokenTree::Punct(_) => {}
                    TokenTree::Literal(lit) => {
                        icon_path.replace(syn::LitStr::new(
                            lit.to_string().as_str(),
                            proc_macro2::Span::call_site(),
                        ));
                    }
                }
            }
            let icon_path = icon_path.expect("invalid icon path");
            let icon_label = icon_label.expect("invalid icon label");
            (icon_path.to_token_stream(), icon_label.to_token_stream())
        } else {
            let value = syn::parse2::<syn::MetaList>(
                proc_macro2::TokenStream::from_str(asset_value.as_str())
                    .expect("value-from-asset-value"),
            )
            .expect("value-parse-error");
            (
                value
                    .parse_args::<syn::MetaNameValue>()
                    .expect("arguments")
                    .value
                    .to_token_stream(),
                proc_macro2::TokenStream::new(),
            )
        };
        let icon_extension = if !is_icon {
            quote::quote!()
        } else {
            quote::quote!(
                    elm.on_fetch(
                        id,
                        #foliage::icon_fetcher!(#foliage::icon::FeatherIcon::#icon_label),
                    );
            )
        };
        let (native, remote) = {
            let native = syn::LitStr::new(
                (native_origin.token().to_string() + res_path.to_string().as_str())
                    .replace("\"", "")
                    .replace("\\", "")
                    .as_str(),
                proc_macro2::Span::call_site(),
            );
            let remote = syn::LitStr::new(
                (remote_origin.token().to_string() + res_path.to_string().as_str())
                    .replace("\"", "")
                    .replace("\\", "")
                    .as_str(),
                proc_macro2::Span::call_site(),
            );
            (native, remote)
        };
        let macro_definition = quote::quote!({
            #[cfg(target_family = "wasm")]
            use #foliage::workflow::Workflow;
            #[cfg(target_family = "wasm")]
            let id = elm.load_remote_asset::<#engen, _>(#remote);
            #foliage::load_native_asset!(elm, id, #native);
            #icon_extension
            id
        });
        groups
            .get_mut(&group_name)
            .expect("group_name-get")
            .add(macro_definition);
    }
    let mut field_iterator = vec![];
    let mut loaders = vec![];
    let mut tys = vec![];
    for group in groups.iter() {
        let f = syn::Ident::new(group.0, proc_macro2::Span::call_site());
        match group.1.ty {
            AssetStorageType::Single => {
                let ty = syn::Ident::new("AssetKey", proc_macro2::Span::call_site());
                let l = group
                    .1
                    .members
                    .first()
                    .expect("first-member-single")
                    .to_token_stream();
                let l = syn::parse2::<syn::Expr>(l).expect("expr-block");
                field_iterator.push(quote::quote!(#f));
                loaders.push(quote::quote!(
                    #l
                ));
                tys.push(quote::quote!(#ty));
            }
            AssetStorageType::Vec => {
                field_iterator.push(quote::quote!(#f));
                let group_members = group
                    .1
                    .members
                    .clone()
                    .iter()
                    .map(|gm| {
                        syn::parse2::<syn::Expr>(gm.to_token_stream()).expect("syn-expr-block")
                    })
                    .collect::<Vec<syn::Expr>>();
                loaders.push(quote::quote!(vec![#(#group_members),*]));
                let ty = proc_macro2::TokenStream::from_str("Vec<AssetKey>").expect("vec-parsing");
                tys.push(quote::quote!(#ty));
            }
        }
    }
    let gen = quote::quote! {
        pub(crate) struct #name {
            #(#field_iterator: #tys),*
        }
        impl #name {
            pub(crate) fn proc_gen_load(elm: &mut #foliage::elm::Elm) -> Self {
                Self {
                    #(#field_iterator: #loaders),*
                }
            }
        }
    };
    gen.into()
}
#[derive(PartialEq, Copy, Clone)]
enum AssetStorageType {
    Single,
    Vec,
}
struct Group {
    ty: AssetStorageType,
    members: Vec<proc_macro2::TokenStream>,
}
impl Group {
    fn new(ty: AssetStorageType) -> Self {
        Self {
            ty,
            members: vec![],
        }
    }
    fn add(&mut self, member: proc_macro2::TokenStream) {
        if self.ty == AssetStorageType::Single && !self.members.is_empty() {
            panic!(
                "attempting to add two assets to the same field.
                Try grouping them with #[group = \"<group-name>\",
                or renaming one of the assets field-name."
            );
        }
        self.members.push(member);
    }
}
fn get_group_value(asset_attributes: &Vec<Attribute>, i: i32) -> String {
    asset_attributes
        .get(i as usize)
        .expect("asset-attribute-get")
        .meta
        .require_name_value()
        .expect("group must have value = \"<name>\"")
        .value
        .to_token_stream()
        .to_string()
}
