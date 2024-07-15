use proc_macro_crate::{crate_name, FoundCrate};
use syn::{parse_macro_input, ItemEnum};

// #[proc_macro_attribute]
// pub fn target_handle(
//     _attrs: proc_macro::TokenStream,
//     input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     let input = parse_macro_input!(input as ItemEnum);
//     let name = &input.ident;
//     let found_crate = crate_name("foliage").expect("foliage is present in `Cargo.toml`");
//     let foliage = match found_crate {
//         FoundCrate::Itself => quote::quote!(crate),
//         FoundCrate::Name(name) => {
//             let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
//             quote::quote!( #ident )
//         }
//     };
//     let gen = quote::quote!(
//         #[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
//         #input
//         impl From<#name> for #foliage::action::TargetHandle {
//             fn from(value: #name) -> Self {
//                 Self(value as i32)
//             }
//         }
//     );
//     gen.into()
// }
//
// #[proc_macro_attribute]
// pub fn action_handle(
//     _attrs: proc_macro::TokenStream,
//     input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     let input = parse_macro_input!(input as ItemEnum);
//     let name = &input.ident;
//     let found_crate = crate_name("foliage").expect("foliage is present in `Cargo.toml`");
//     let foliage = match found_crate {
//         FoundCrate::Itself => quote::quote!(crate),
//         FoundCrate::Name(name) => {
//             let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
//             quote::quote!( #ident )
//         }
//     };
//     let gen = quote::quote!(
//         #[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
//         #input
//         impl From<#name> for #foliage::action::ActionHandle {
//             fn from(value: #name) -> Self {
//                 Self(value as i32)
//             }
//         }
//     );
//     gen.into()
// }
