use proc_macro_crate::{FoundCrate, crate_name};
use syn::{Fields, Item, ItemEnum, ItemStruct, parse_macro_input};

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

/// Shared machinery for `#[component]`/`#[resource]`/`#[query_data]`/`#[system_set]`: none
/// of bevy_ecs's own derives can be told to emit fully-qualified paths -- their path
/// resolution (`bevy_ecs_path()`, checking the *consuming* crate's own `Cargo.toml` for a
/// direct `bevy_ecs`/`bevy`/`bevy_internal` dependency) is baked into bevy_ecs's own macro
/// implementation, independent of how the derive was named at the call site. So simply
/// invoking `#[derive(::foliage::bevy_ecs::component::Component)]` directly doesn't help --
/// bevy_ecs's generated `impl` would still reference the bare, unqualified name `bevy_ecs`,
/// which a consumer crate (depending only on `foliage`, not `bevy_ecs` directly) has no
/// reason to have in scope.
///
/// Instead of reimplementing any of these (`QueryData` in particular is `unsafe trait
/// QueryData: WorldQuery` -- hand-rolling it would be a soundness risk, not just a
/// convenience gap), this nests the annotated item inside a uniquely-named, otherwise-empty
/// module that binds `bevy_ecs` locally via `use`, invokes the *real* derive there (where it
/// now resolves correctly), then re-exports the item back out at its original visibility.
/// Delegates entirely to bevy_ecs's real, tested derive logic.
///
/// Two simpler alternatives were tried and both broke: a bare (non-nested) sibling `use
/// bevy_ecs;` collides (`E0252`) the moment `#[component]`/etc. is used more than once in
/// the same file, since every invocation would import the same name; uniquely *aliasing*
/// that import per invocation avoids the collision but then `#[derive(alias::Component)]`
/// fails to resolve the macro at all -- derive-macro-path resolution doesn't see a sibling
/// `use` alias from the same expansion the way plain item paths do. The module is the one
/// approach that's actually been verified to compile, including with several `#[component]`
/// structs in one file.
fn wrap_bevy_derive(
    input: proc_macro::TokenStream,
    derive_path: proc_macro2::TokenStream,
) -> proc_macro::TokenStream {
    let mut item = parse_macro_input!(input as Item);
    let root = foliage_root();
    // A field written with no visibility keyword is private to its *defining module* --
    // moving the struct one level deeper (into `#scope` below) would silently shrink that
    // to private-to-the-hidden-module, breaking access from code that used to be a sibling.
    // `pub(in super)` restores the exact original meaning: visible to the module that
    // actually declared it, and nothing wider. Fields with an explicit visibility (`pub`,
    // `pub(crate)`, ..) are untouched -- already an absolute statement of intent that the
    // extra nesting doesn't change.
    fn widen_inherited_field_vis(fields: &mut Fields) {
        for field in fields.iter_mut() {
            if matches!(field.vis, syn::Visibility::Inherited) {
                field.vis = syn::parse_quote!(pub(in super));
            }
        }
    }
    let (vis, ident) = match &mut item {
        Item::Struct(s) => {
            let vis = s.vis.clone();
            s.vis = syn::parse_quote!(pub);
            widen_inherited_field_vis(&mut s.fields);
            (vis, s.ident.clone())
        }
        Item::Enum(e) => {
            let vis = e.vis.clone();
            e.vis = syn::parse_quote!(pub);
            (vis, e.ident.clone())
        }
        _ => {
            return syn::Error::new_spanned(&item, "expected a struct or enum")
                .to_compile_error()
                .into();
        }
    };
    let scope = syn::Ident::new(
        &format!("__foliage_scope_{ident}"),
        proc_macro2::Span::call_site(),
    );
    let expanded = quote::quote! {
        #[allow(non_snake_case)]
        mod #scope {
            use super::*;
            use #root::bevy_ecs;
            #[derive(#derive_path)]
            #item
        }
        #vis use #scope::#ident;
    };
    expanded.into()
}

/// `#[derive(Component)]`, without needing `bevy_ecs` in scope. Supports the same
/// `#[component(..)]`/`#[require(..)]` attribute surface as the real derive -- those pass
/// through untouched, since this only changes *where* the derive runs, not what it does.
#[proc_macro_attribute]
pub fn component(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_bevy_derive(input, quote::quote!(bevy_ecs::component::Component))
}

/// `#[derive(Resource)]`, without needing `bevy_ecs` in scope.
#[proc_macro_attribute]
pub fn resource(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_bevy_derive(input, quote::quote!(bevy_ecs::resource::Resource))
}

/// `#[derive(QueryData)]`, without needing `bevy_ecs` in scope -- for bundling several
/// query terms into one named, field-accessed system parameter instead of a positional
/// tuple: `fn sys(items: Query<MyQuery>)` instead of `Query<(Entity, &A, &mut B)>`.
#[proc_macro_attribute]
pub fn query_data(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_bevy_derive(input, quote::quote!(bevy_ecs::query::QueryData))
}

/// `#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]`, without needing `bevy_ecs` in
/// scope -- the trailing four are bevy's own required bound on every `SystemSet`, always
/// needed together, so this includes them rather than making every caller repeat them.
#[proc_macro_attribute]
pub fn system_set(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_bevy_derive(
        input,
        quote::quote!(
            bevy_ecs::schedule::SystemSet,
            Debug,
            Clone,
            PartialEq,
            Eq,
            Hash
        ),
    )
}

/// Turns a plain struct into a targeted event: injects the `entity: Entity` plumbing
/// field, implements bevy's `Event`/`EntityEvent` (generated directly with fully-qualified
/// paths, so consumer crates need no `bevy_ecs` dependency for the derive's manifest
/// resolution), derives `Clone`, implements `TargetedEvent`, and generates `new(<fields>)`
/// with the target prefilled (`Entity::PLACEHOLDER` — the send seam assigns the real
/// target). The author writes only their payload:
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
    let found_crate = crate_name("foliage").expect("foliage is present in `Cargo.toml`");
    let foliage = match found_crate {
        FoundCrate::Itself => quote::quote!(crate),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote::quote!( #ident )
        }
    };
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
