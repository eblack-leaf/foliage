use proc_macro_crate::{FoundCrate, crate_name};
use syn::{Fields, ItemEnum, ItemStruct, parse_macro_input};

/// Path to the foliage crate root from wherever the macro is invoked: `foliage` for
/// consumers, `crate` inside foliage_proper's (or foliage's) own *library* target.
///
/// `proc_macro_crate::crate_name`'s `FoundCrate::Itself` means "the queried name matches
/// `CARGO_PKG_NAME`" -- true for *every* target in that package (the lib, but also every
/// example/bin/test), not just the lib. `crate::bevy_ecs` only actually resolves from the
/// lib target itself; an example (a separate compilation unit that only sees the package's
/// *public* API, the same as any external consumer would) needs the real crate name even
/// though `crate_name` reports `Itself`. `CARGO_CRATE_NAME` (the specific target currently
/// being compiled) vs `CARGO_PKG_NAME` (the package, constant across all its targets)
/// distinguishes the two: they match only for the lib target itself. Found by actually
/// hitting `error[E0432]: unresolved import 'crate', no 'bevy_ecs' in the root` compiling
/// `foliage/examples/polyline.rs` -- `Itself` alone isn't a safe signal on its own.
fn foliage_root() -> proc_macro2::TokenStream {
    let found = crate_name("foliage").or_else(|_| crate_name("foliage_proper"));
    match found {
        Ok(FoundCrate::Itself) if compiling_the_actual_lib_target() => quote::quote!(crate),
        Ok(FoundCrate::Itself) => {
            // Itself, but not the lib target -- fall back to the package's own name
            // (CARGO_PKG_NAME), same as a downstream consumer would spell it.
            let name = std::env::var("CARGO_PKG_NAME")
                .unwrap_or_else(|_| "foliage".to_string())
                .replace('-', "_");
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote::quote!( #ident )
        }
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote::quote!( #ident )
        }
        Err(_) => quote::quote!(foliage),
    }
}

fn compiling_the_actual_lib_target() -> bool {
    let crate_name = std::env::var("CARGO_CRATE_NAME").unwrap_or_default();
    let pkg_name = std::env::var("CARGO_PKG_NAME")
        .unwrap_or_default()
        .replace('-', "_");
    crate_name == pkg_name
}

/// Turns a plain struct into a targeted event: injects the `entity: Entity` plumbing
/// field, implements bevy's `Event`/`EntityEvent` (generated directly with fully-qualified
/// paths, so the derive's own manifest resolution never runs), derives `Clone`, implements
/// `TargetedEvent`, and generates `new(<fields>)` with the target prefilled
/// (`Entity::PLACEHOLDER` — the send seam assigns the real target). The author writes only
/// their payload:
///
/// ```ignore
/// #[foliage_macros::targeted_event]
/// pub struct CardPlayed { pub id: u32 }
/// // emit:  tree.trigger_targets(CardPlayed::new(id), target);
/// ```
///
/// Engine-internal, and not re-exported from `foliage`: the `TargetedEvent` it implements is
/// `pub(crate)` in `foliage_proper`, so this only ever resolves from inside the engine. Apps
/// are on the other side of the boundary and send nothing shaped like this.
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
    let expanded = quote::quote!(
        #[derive(Clone)]
        #input
        impl #root::bevy_ecs::event::Event for #name {
            type Trigger<'a> = #root::bevy_ecs::event::EntityTrigger;
        }
        impl #root::bevy_ecs::event::EntityEvent for #name {
            fn event_target(&self) -> #root::bevy_ecs::entity::Entity {
                self.entity
            }
        }
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
            /// The entity this event targets. Inherent (reachable through the trigger's
            /// deref), so observer bodies need no `EntityEvent` trait import.
            pub fn event_target(&self) -> #root::bevy_ecs::entity::Entity {
                self.entity
            }
        }
    );
    expanded.into()
}

#[proc_macro_attribute]
pub fn icon_handle(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    let name = &input.ident;
    // Was its own separate, un-deduplicated copy of this same crate-path resolution --
    // carried the exact same `Itself`-means-lib-target bug `foliage_root()` had until an
    // example in this same package (`foliage/examples/controls.rs`, using `#[icon_handle]`)
    // hit it too. Sharing the one (now-fixed) helper instead of maintaining two copies.
    let foliage = foliage_root();
    let expanded = quote::quote!(
        #[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
        #input
        impl From<#name> for #foliage::IconId {
            fn from(value: #name) -> #foliage::IconId {
                value as #foliage::IconId
            }
        }
    );
    expanded.into()
}
