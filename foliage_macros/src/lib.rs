use proc_macro2::TokenTree;
use proc_macro_crate::{crate_name, FoundCrate};
use syn::Meta;

#[proc_macro_derive(SceneBinding)]
pub fn scene_binding_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let ast: syn::DeriveInput = syn::parse2(input).unwrap();
    let name = &ast.ident;
    let found_crate = crate_name("foliage").expect("foliage is present in `Cargo.toml`");
    let scene_binding = match found_crate {
        FoundCrate::Itself => quote::quote!(crate::scene::SceneBinding),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote::quote!( #ident::scene::SceneBinding )
        }
    };
    let gen = match &ast.data {
        syn::Data::Enum(_) => {
            quote::quote! {
                impl From<#name> for #scene_binding {
                    fn from(value: #name) -> #scene_binding {
                        #scene_binding(value as i32)
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
    let modified_input = proc_macro2::TokenStream::from(input);
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
    let ast: syn::DeriveInput = syn::parse2(modified_input).unwrap();
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
        syn::Data::Struct(parsed_struct) => {
            let mut fields = vec![];
            let mut loaders = vec![];
            let mut tys = vec![];
            for field in parsed_struct.fields.iter() {
                let field_name = &field.ident;
                fields.push(quote::quote!(#field_name));
                let field_type = &field.ty;
                tys.push(quote::quote!(#field_type));
                if let Some(attr) = field.attrs.first() {
                    match &attr.meta {
                        Meta::Path(_) => {}
                        Meta::List(ml) => {
                            if ml.path.require_ident().unwrap().to_string().as_str() == "icon" {
                                let mut value = None;
                                let mut icon_label = None;
                                for t in ml.tokens.clone() {
                                    match t {
                                        TokenTree::Group(_) => {}
                                        TokenTree::Ident(id) => {
                                            icon_label.replace(syn::Ident::new(
                                                id.to_string().as_str(),
                                                proc_macro2::Span::call_site(),
                                            ));
                                        }
                                        TokenTree::Punct(_) => {}
                                        TokenTree::Literal(lit) => {
                                            value.replace(syn::LitStr::new(
                                                lit.to_string().as_str(),
                                                proc_macro2::Span::call_site(),
                                            ));
                                        }
                                    }
                                }
                                let value = value.unwrap();
                                let icon_label = icon_label.unwrap();
                                let native = syn::LitStr::new(
                                    (native_origin.token().to_string() + value.value().as_str())
                                        .replace("\"", "")
                                        .replace("\\", "")
                                        .as_str(),
                                    proc_macro2::Span::call_site(),
                                );
                                let remote = syn::LitStr::new(
                                    (remote_origin.token().to_string() + value.value().as_str())
                                        .replace("\"", "")
                                        .replace("\\", "")
                                        .as_str(),
                                    proc_macro2::Span::call_site(),
                                );
                                loaders.push(quote::quote!(
                                    {
                                        #[cfg(target_family = "wasm")]
                                        use #foliage::workflow::Workflow;
                                        #[cfg(target_family = "wasm")]
                                        let id = elm.load_remote_asset::<#engen, _>(
                                            #remote
                                        );
                                        #[cfg(not(target_family = "wasm"))]
                                        #foliage::load_native_asset!(elm, id, #native);
                                        elm.on_fetch(
                                            id,
                                            #foliage::icon_fetcher!(#foliage::icon::FeatherIcon::#icon_label)
                                        );
                                        id
                                    }
                                ));
                            } else {
                                for t in ml.tokens.clone() {
                                    match t {
                                        TokenTree::Literal(lit) => {
                                            let native = syn::LitStr::new(
                                                (native_origin.token().to_string()
                                                    + lit.to_string().as_str())
                                                .replace("\"", "")
                                                .replace("\\", "")
                                                .as_str(),
                                                proc_macro2::Span::call_site(),
                                            );
                                            let remote = syn::LitStr::new(
                                                (remote_origin.token().to_string()
                                                    + lit.to_string().as_str())
                                                .replace("\"", "")
                                                .replace("\\", "")
                                                .as_str(),
                                                proc_macro2::Span::call_site(),
                                            );
                                            loaders.push(quote::quote!(
                                                {
                                                    #[cfg(target_family = "wasm")]
                                                    use #foliage::workflow::Workflow;
                                                    #[cfg(target_family = "wasm")]
                                                    let id = elm.load_remote_asset::<#engen, _>(#remote);
                                                    #foliage::load_native_asset!(elm, id, #native);
                                                    id
                                                }
                                            ));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        Meta::NameValue(_) => {}
                    }
                }
            }
            quote::quote! {
                pub(crate) struct #name {
                    #(#fields: #tys),*
                }
                impl #name {
                    pub(crate) fn proc_gen_load(elm: &mut #foliage::elm::Elm) -> Self {
                        Self {
                            #(#fields: #loaders),*
                        }
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