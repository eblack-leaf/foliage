use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
#[proc_macro_derive(SceneBinding)]
pub fn scene_binding_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
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
