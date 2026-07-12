use proc_macro_crate::{crate_name, FoundCrate};
use syn::{parse_macro_input, Fields, ItemEnum, ItemStruct};

/// Path to the foliage crate root from wherever the macro is invoked: `foliage` for
/// consumers, `crate` inside foliage_proper itself.
fn foliage_root() -> proc_macro2::TokenStream {
    let found = crate_name("foliage").or_else(|_| crate_name("foliage_proper"));
    match found {
        Ok(FoundCrate::Itself) => quote::quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote::quote!( #ident )
        }
        Err(_) => quote::quote!(foliage),
    }
}

/// Turns a plain struct into a targeted event: injects the `entity: Entity` plumbing
/// field, derives bevy's `EntityEvent` + `Clone`, implements `TargetedEvent`, and
/// generates `new(<fields>)` with the target prefilled (`Entity::PLACEHOLDER` — the
/// send seam assigns the real target). The author writes only their payload:
///
/// ```ignore
/// #[targeted_event]
/// pub struct CardPlayed { pub id: u32 }
/// // emit:  tree.trigger_targets(CardPlayed::new(id), target);
/// ```
#[proc_macro_attribute]
pub fn targeted_event(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as ItemStruct);
    let root = foliage_root();
    let name = input.ident.clone();
    let user_fields: Vec<(syn::Ident, syn::Type)> = match &input.fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|f| (f.ident.clone().unwrap(), f.ty.clone()))
            .collect(),
        Fields::Unit => Vec::new(),
        Fields::Unnamed(_) => {
            return syn::Error::new_spanned(
                &input,
                "targeted_event requires named fields (or a unit struct)",
            )
            .to_compile_error()
            .into();
        }
    };
    let entity_field: syn::Field = syn::parse_quote! { entity: #root::bevy_ecs::entity::Entity };
    let mut named = syn::punctuated::Punctuated::new();
    named.push(entity_field);
    if let Fields::Named(existing) = &input.fields {
        for f in existing.named.iter() {
            named.push(f.clone());
        }
    }
    input.fields = Fields::Named(syn::FieldsNamed {
        brace_token: Default::default(),
        named,
    });
    let args = user_fields
        .iter()
        .map(|(id, ty)| quote::quote!(#id: impl Into<#ty>));
    let inits = user_fields
        .iter()
        .map(|(id, _)| quote::quote!(#id: #id.into()));
    let gen = quote::quote!(
        #[derive(#root::bevy_ecs::event::EntityEvent, Clone)]
        #input
        impl #root::TargetedEvent for #name {
            fn set_target(&mut self, entity: #root::bevy_ecs::entity::Entity) {
                self.entity = entity;
            }
        }
        impl #name {
            pub fn new(#(#args),*) -> Self {
                Self {
                    entity: #root::bevy_ecs::entity::Entity::PLACEHOLDER,
                    #(#inits),*
                }
            }
        }
    );
    gen.into()
}

#[proc_macro_attribute]
pub fn icon_handle(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    let name = &input.ident;
    let found_crate = crate_name("foliage").expect("foliage is present in `Cargo.toml`");
    let foliage = match found_crate {
        FoundCrate::Itself => quote::quote!(crate),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote::quote!( #ident )
        }
    };
    let gen = quote::quote!(
        #[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
        #input
        impl From<#name> for #foliage::IconId {
            fn from(value: #name) -> #foliage::IconId {
                value as #foliage::IconId
            }
        }
    );
    gen.into()
}
