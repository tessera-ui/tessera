//! # Tessera Macros
//!
//! This crate provides procedural macros for the Tessera UI framework.
//! The main export is the `#[tessera]` attribute macro, which transforms
//! regular Rust functions into Tessera UI components.

use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Block, Data, DeriveInput, Expr, Fields, FnArg, GenericArgument, Ident, ItemFn, Pat, Path,
    PathArguments, Token, Type, parse::Parse, parse_macro_input, parse_quote, visit_mut::VisitMut,
};

/// Helper: parse crate path from attribute TokenStream
fn parse_crate_path(attr: proc_macro::TokenStream) -> syn::Result<syn::Path> {
    if attr.is_empty() {
        // Default to `tessera_ui` if no path is provided
        Ok(syn::parse_quote!(::tessera_ui))
    } else {
        // Parse the provided path, e.g., `crate` or `tessera_ui`
        let tokens: proc_macro2::TokenStream = attr.clone().into();
        syn::parse(attr).map_err(|_| {
            syn::Error::new_spanned(tokens, "expected a crate path like `crate` or `tessera_ui`")
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct SetterAttrConfig {
    skip: bool,
    into: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PropHelperKind {
    Callback,
    CallbackWith,
    RenderSlot,
    RenderSlotWith,
}

#[derive(Clone, Copy, Debug, Default)]
struct PropFieldAttrConfig {
    skip_eq: bool,
}

#[derive(Default)]
struct PropContainerAttrConfig {
    crate_path: Option<Path>,
}

fn parse_setter_attr(attrs: &[syn::Attribute]) -> syn::Result<SetterAttrConfig> {
    let mut config = SetterAttrConfig::default();
    for attr in attrs {
        if !attr.path().is_ident("prop") {
            continue;
        }

        match &attr.meta {
            syn::Meta::Path(_) => {}
            syn::Meta::List(_) => {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        return Err(meta.error("unsupported setter option `skip`"));
                    }
                    if meta.path.is_ident("skip_setter") {
                        config.skip = true;
                        return Ok(());
                    }
                    if meta.path.is_ident("into") {
                        config.into = true;
                        return Ok(());
                    }
                    Ok(())
                })?;
            }
            syn::Meta::NameValue(_) => {
                return Err(syn::Error::new_spanned(
                    attr,
                    "unsupported #[prop = ...] form; expected #[prop(...)]",
                ));
            }
        }
    }
    Ok(config)
}

fn merge_setter_attr(container: SetterAttrConfig, field: SetterAttrConfig) -> SetterAttrConfig {
    SetterAttrConfig {
        skip: container.skip || field.skip,
        into: container.into || field.into,
    }
}

fn parse_prop_field_attr(attrs: &[syn::Attribute]) -> syn::Result<PropFieldAttrConfig> {
    let mut config = PropFieldAttrConfig::default();
    for attr in attrs {
        if !attr.path().is_ident("prop") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip_eq") {
                if config.skip_eq {
                    return Err(meta.error("duplicate `skip_eq` in #[prop(...)]"));
                }
                config.skip_eq = true;
                return Ok(());
            }
            if meta.path.is_ident("skip") {
                return Err(meta.error("unsupported field option `skip`"));
            }
            if meta.path.is_ident("skip_setter") || meta.path.is_ident("into") {
                return Ok(());
            }
            if meta.path.is_ident("crate_path") {
                return Err(meta.error(
                    "container option in field #[prop(...)]; `crate_path` is only valid on structs",
                ));
            }

            Err(meta.error(
                "unsupported field #[prop(...)] option; expected setter options (`skip_setter`/`into`) or compare option (`skip_eq`)",
            ))
        })?;
    }
    Ok(config)
}

fn parse_prop_container_attr(attrs: &[syn::Attribute]) -> syn::Result<PropContainerAttrConfig> {
    let mut config = PropContainerAttrConfig::default();
    for attr in attrs {
        if !attr.path().is_ident("prop") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate_path") {
                if config.crate_path.is_some() {
                    return Err(meta.error("duplicate `crate_path` in #[prop(...)]"));
                }
                let value = meta.value()?;
                config.crate_path = Some(value.parse::<Path>()?);
                return Ok(());
            }
            if meta.path.is_ident("skip") {
                return Err(meta.error("unsupported struct option `skip`"));
            }
            if meta.path.is_ident("skip_setter") {
                return Ok(());
            }
            if meta.path.is_ident("into") {
                return Err(meta.error(
                    "unsupported struct option `into`; use `#[prop(into)]` on fields",
                ));
            }
            if meta.path.is_ident("no_setters") {
                return Err(meta.error(
                    "unsupported struct option `no_setters`; use `#[prop(skip_setter)]` instead",
                ));
            }
            if meta.path.is_ident("skip_eq") {
                return Err(meta.error(
                    "field option in struct #[prop(...)]; `skip_eq` is only valid on fields",
                ));
            }
            Err(meta.error(
                "unsupported struct #[prop(...)] option; expected `crate_path = ...` or `skip_setter`",
            ))
        })?;
    }
    Ok(config)
}

fn option_inner_type(ty: &Type) -> Option<Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    arguments.args.iter().find_map(|arg| match arg {
        GenericArgument::Type(inner) => Some(inner.clone()),
        _ => None,
    })
}

fn parse_functor_signature(ty: &Type, type_name: &str) -> Option<(Type, Type)> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != type_name {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    let mut types = arguments.args.iter().filter_map(|arg| match arg {
        GenericArgument::Type(ty) => Some(ty.clone()),
        _ => None,
    });
    let arg = types.next()?;
    let ret = types.next().unwrap_or_else(|| parse_quote!(()));
    Some((arg, ret))
}

fn type_last_segment_ident(ty: &Type) -> Option<&Ident> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    type_path.path.segments.last().map(|segment| &segment.ident)
}

fn infer_prop_helper_kind(ty: &Type) -> Option<PropHelperKind> {
    let value_ty = option_inner_type(ty).unwrap_or_else(|| ty.clone());
    let ident = type_last_segment_ident(&value_ty)?;
    if ident == "Callback" {
        return Some(PropHelperKind::Callback);
    }
    if ident == "CallbackWith" {
        return Some(PropHelperKind::CallbackWith);
    }
    if ident == "RenderSlot" {
        return Some(PropHelperKind::RenderSlot);
    }
    if ident == "RenderSlotWith" {
        return Some(PropHelperKind::RenderSlotWith);
    }
    None
}

fn is_arc_type(ty: &Type) -> bool {
    type_last_segment_ident(ty).is_some_and(|ident| ident == "Arc")
}

fn is_rc_type(ty: &Type) -> bool {
    type_last_segment_ident(ty).is_some_and(|ident| ident == "Rc")
}

fn field_compare_expr(field_ident: &Ident, ty: &Type) -> proc_macro2::TokenStream {
    if is_arc_type(ty) {
        return quote! { ::std::sync::Arc::ptr_eq(&self.#field_ident, &other.#field_ident) };
    }
    if is_rc_type(ty) {
        return quote! { ::std::rc::Rc::ptr_eq(&self.#field_ident, &other.#field_ident) };
    }
    if let Some(inner_ty) = option_inner_type(ty) {
        if is_arc_type(&inner_ty) {
            return quote! {
                match (&self.#field_ident, &other.#field_ident) {
                    (Some(lhs), Some(rhs)) => ::std::sync::Arc::ptr_eq(lhs, rhs),
                    (None, None) => true,
                    _ => false,
                }
            };
        }
        if is_rc_type(&inner_ty) {
            return quote! {
                match (&self.#field_ident, &other.#field_ident) {
                    (Some(lhs), Some(rhs)) => ::std::rc::Rc::ptr_eq(lhs, rhs),
                    (None, None) => true,
                    _ => false,
                }
            };
        }
    }
    quote! { self.#field_ident == other.#field_ident }
}

#[cfg(feature = "shard")]
#[derive(Default)]
struct ShardMacroArgs {
    crate_path: Option<Path>,
    state_type: Option<Type>,
    lifecycle: Option<Ident>,
}

#[cfg(feature = "shard")]
struct ShardParam {
    ident: Ident,
    ty: Type,
}

#[cfg(feature = "shard")]
fn parse_shard_params(sig: &syn::Signature) -> syn::Result<Vec<ShardParam>> {
    let mut params = Vec::with_capacity(sig.inputs.len());
    for arg in &sig.inputs {
        match arg {
            FnArg::Receiver(receiver) => {
                return Err(syn::Error::new_spanned(
                    receiver,
                    "#[shard] does not support methods; use a free function",
                ));
            }
            FnArg::Typed(pat_type) => {
                let Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
                    return Err(syn::Error::new_spanned(
                        &pat_type.pat,
                        "#[shard] parameters must be simple named bindings like `foo: T`",
                    ));
                };
                params.push(ShardParam {
                    ident: pat_ident.ident.clone(),
                    ty: (*pat_type.ty).clone(),
                });
            }
        }
    }
    Ok(params)
}

#[cfg(feature = "shard")]
impl Parse for ShardMacroArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let mut args = ShardMacroArgs::default();
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "crate_path" => {
                    if args.crate_path.is_some() {
                        return Err(syn::Error::new(
                            key.span(),
                            "duplicate `crate_path` argument",
                        ));
                    }
                    args.crate_path = Some(input.parse::<Path>()?);
                }
                "state" => {
                    if args.state_type.is_some() {
                        return Err(syn::Error::new(key.span(), "duplicate `state` argument"));
                    }
                    args.state_type = Some(input.parse::<Type>()?);
                }
                "lifecycle" => {
                    if args.lifecycle.is_some() {
                        return Err(syn::Error::new(
                            key.span(),
                            "duplicate `lifecycle` argument",
                        ));
                    }
                    args.lifecycle = Some(input.parse::<Ident>()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "unsupported #[shard(...)] argument; expected `state`, `lifecycle`, or `crate_path`",
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(args)
    }
}

/// Helper: tokens to register a component node
fn register_node_tokens(crate_path: &syn::Path, fn_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        {
            use #crate_path::ComponentNode;
            use #crate_path::layout::DefaultLayoutSpec;
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime.component_tree.add_node(
                    ComponentNode {
                        fn_name: stringify!(#fn_name).to_string(),
                        component_type_id: __tessera_component_type_id,
                        instance_logic_id: 0,
                        instance_key: 0,
                        pointer_preview_handler_fn: None,
                        pointer_handler_fn: None,
                        pointer_final_handler_fn: None,
                        keyboard_preview_handler_fn: None,
                        keyboard_handler_fn: None,
                        ime_preview_handler_fn: None,
                        ime_handler_fn: None,
                        focus_requester_binding: None,
                        focus_registration: None,
                        focus_restorer_fallback: None,
                        focus_traversal_policy: None,
                        focus_changed_handler: None,
                        focus_event_handler: None,
                        focus_beyond_bounds_handler: None,
                        focus_reveal_handler: None,
                        layout_spec: Box::new(DefaultLayoutSpec::default()),
                        replay: None,
                        props_unchanged_from_previous: false,
                    }
                )
            })
        }
    }
}

/// Parse and validate strict component props signature:
/// `fn component(<prop>: &T)`.
enum ComponentPropSignature {
    Unit,
    RefArg {
        ident: syn::Ident,
        ty: Box<syn::Type>,
    },
}

impl ComponentPropSignature {
    fn prop_type(&self) -> syn::Type {
        match self {
            Self::Unit => syn::parse_quote!(()),
            Self::RefArg { ty, .. } => ty.as_ref().clone(),
        }
    }
}

fn strict_prop_signature(sig: &syn::Signature) -> Result<ComponentPropSignature, syn::Error> {
    if sig.inputs.is_empty() {
        return Ok(ComponentPropSignature::Unit);
    }

    if sig.inputs.len() != 1 {
        return Err(syn::Error::new_spanned(
            &sig.inputs,
            "#[tessera] components must have signature `fn name()` or `fn name(<prop>: &T)`",
        ));
    }

    let Some(arg) = sig.inputs.first() else {
        return Err(syn::Error::new_spanned(
            &sig.inputs,
            "#[tessera] components must have signature `fn name()` or `fn name(<prop>: &T)`",
        ));
    };
    let FnArg::Typed(arg) = arg else {
        return Err(syn::Error::new_spanned(
            arg,
            "#[tessera] methods are not supported; use free functions with `&T` props",
        ));
    };

    let Pat::Ident(pat_ident) = arg.pat.as_ref() else {
        return Err(syn::Error::new_spanned(
            &arg.pat,
            "component parameter must be a named identifier",
        ));
    };

    let Type::Reference(type_ref) = arg.ty.as_ref() else {
        return Err(syn::Error::new_spanned(
            &arg.ty,
            "component parameter must be a shared reference `&T`",
        ));
    };

    if type_ref.mutability.is_some() {
        return Err(syn::Error::new_spanned(
            type_ref,
            "component parameter must be an immutable reference `&T`",
        ));
    }

    Ok(ComponentPropSignature::RefArg {
        ident: pat_ident.ident.clone(),
        ty: Box::new(type_ref.elem.as_ref().clone()),
    })
}

/// Helper: tokens to attach replay metadata to the current component node.
fn replay_register_tokens(
    crate_path: &syn::Path,
    fn_name: &syn::Ident,
    signature: &ComponentPropSignature,
) -> proc_macro2::TokenStream {
    match signature {
        ComponentPropSignature::RefArg { ident, ty } => {
            let ty = ty.as_ref();
            quote! {
                let __tessera_component_reused = {
                    use #crate_path::runtime::TesseraRuntime;
                    let __tessera_runner = #crate_path::prop::make_component_runner::<#ty>(#fn_name);
                    TesseraRuntime::with_mut(|runtime| {
                        runtime.set_current_component_replay(__tessera_runner, #ident)
                    })
                };
            }
        }
        ComponentPropSignature::Unit => quote! {
            let __tessera_component_reused = {
                use #crate_path::runtime::TesseraRuntime;
                fn __tessera_noarg_runner(_props: &()) {
                    #fn_name();
                }
                let __tessera_runner =
                    #crate_path::prop::make_component_runner::<()>(__tessera_noarg_runner);
                let __tessera_unit_props = ();
                TesseraRuntime::with_mut(|runtime| {
                    runtime.set_current_component_replay(__tessera_runner, &__tessera_unit_props)
                })
            };
        },
    }
}

/// Helper: compile-time assertion that component props implement `Prop`.
fn prop_assert_tokens(crate_path: &syn::Path, prop_type: &syn::Type) -> proc_macro2::TokenStream {
    quote! {
        {
            fn __tessera_assert_prop<T: #crate_path::Prop>() {}
            __tessera_assert_prop::<#prop_type>();
        }
    }
}

/// Helper: tokens to inject `layout`
fn layout_inject_tokens(crate_path: &syn::Path) -> proc_macro2::TokenStream {
    quote! {
        #[allow(clippy::needless_pass_by_value)]
        fn layout<S>(spec: S)
        where
            S: #crate_path::layout::LayoutSpec,
        {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| runtime.set_current_layout_spec(spec));
        }
    }
}

/// Helper: tokens to inject typed input handlers.
fn input_handler_inject_tokens(crate_path: &syn::Path) -> proc_macro2::TokenStream {
    quote! {
        #[allow(clippy::needless_pass_by_value)]
        fn pointer_preview_input_handler<F>(fun: F)
        where
            F: Fn(#crate_path::PointerInput) + Send + Sync + 'static,
        {
            use #crate_path::PointerInputHandlerFn;
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .current_node_mut()
                    .unwrap()
                    .pointer_preview_handler_fn = Some(Box::new(fun) as Box<PointerInputHandlerFn>)
            });
        }

        #[allow(clippy::needless_pass_by_value)]
        fn pointer_input_handler<F>(fun: F)
        where
            F: Fn(#crate_path::PointerInput) + Send + Sync + 'static,
        {
            use #crate_path::PointerInputHandlerFn;
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .current_node_mut()
                    .unwrap()
                    .pointer_handler_fn = Some(Box::new(fun) as Box<PointerInputHandlerFn>)
            });
        }

        #[allow(clippy::needless_pass_by_value)]
        fn pointer_final_input_handler<F>(fun: F)
        where
            F: Fn(#crate_path::PointerInput) + Send + Sync + 'static,
        {
            use #crate_path::PointerInputHandlerFn;
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .current_node_mut()
                    .unwrap()
                    .pointer_final_handler_fn = Some(Box::new(fun) as Box<PointerInputHandlerFn>)
            });
        }

        #[allow(clippy::needless_pass_by_value)]
        fn keyboard_preview_input_handler<F>(fun: F)
        where
            F: Fn(#crate_path::KeyboardInput) + Send + Sync + 'static,
        {
            use #crate_path::KeyboardInputHandlerFn;
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .current_node_mut()
                    .unwrap()
                    .keyboard_preview_handler_fn = Some(Box::new(fun) as Box<KeyboardInputHandlerFn>)
            });
        }

        #[allow(clippy::needless_pass_by_value)]
        fn keyboard_input_handler<F>(fun: F)
        where
            F: Fn(#crate_path::KeyboardInput) + Send + Sync + 'static,
        {
            use #crate_path::KeyboardInputHandlerFn;
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .current_node_mut()
                    .unwrap()
                    .keyboard_handler_fn = Some(Box::new(fun) as Box<KeyboardInputHandlerFn>)
            });
        }

        #[allow(clippy::needless_pass_by_value)]
        fn ime_preview_input_handler<F>(fun: F)
        where
            F: Fn(#crate_path::ImeInput) + Send + Sync + 'static,
        {
            use #crate_path::ImeInputHandlerFn;
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .current_node_mut()
                    .unwrap()
                    .ime_preview_handler_fn = Some(Box::new(fun) as Box<ImeInputHandlerFn>)
            });
        }

        #[allow(clippy::needless_pass_by_value)]
        fn ime_input_handler<F>(fun: F)
        where
            F: Fn(#crate_path::ImeInput) + Send + Sync + 'static,
        {
            use #crate_path::ImeInputHandlerFn;
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .current_node_mut()
                    .unwrap()
                    .ime_handler_fn = Some(Box::new(fun) as Box<ImeInputHandlerFn>)
            });
        }

        #[allow(clippy::needless_pass_by_value)]
        fn focus_requester_with(requester: #crate_path::FocusRequester) {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime.bind_current_focus_requester(requester);
            });
        }

        fn focus_requester() -> #crate_path::FocusRequester {
            #crate_path::runtime::persistent_focus_requester_for_current_instance(
                "__tessera_focus_requester",
            )
        }

        fn remember_focus_requester() -> #crate_path::FocusRequester {
            focus_requester()
        }

        fn focus_target_handle() -> #crate_path::FocusNode {
            #crate_path::runtime::persistent_focus_target_for_current_instance(
                "__tessera_focus_target",
            )
        }

        fn focus_scope_handle() -> #crate_path::FocusScopeNode {
            #crate_path::runtime::persistent_focus_scope_for_current_instance(
                "__tessera_focus_scope",
            )
        }

        fn remember_focus_scope() -> #crate_path::FocusScopeNode {
            focus_scope_handle()
        }

        fn focus_group_handle() -> #crate_path::FocusGroupNode {
            #crate_path::runtime::persistent_focus_group_for_current_instance(
                "__tessera_focus_group",
            )
        }

        fn remember_focus_group() -> #crate_path::FocusGroupNode {
            focus_group_handle()
        }

        #[allow(clippy::needless_pass_by_value)]
        fn focus_target_with(node: #crate_path::FocusNode) -> #crate_path::FocusNode {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime.register_current_focus_target(node);
            });
            node
        }

        fn focus_target() -> #crate_path::FocusNode {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                if let Some(node) = runtime.current_focus_target_handle() {
                    node
                } else {
                    let node = focus_target_handle();
                    runtime.ensure_current_focus_target(node);
                    node
                }
            })
        }

        fn focusable() -> #crate_path::FocusNode {
            focus_target()
        }

        #[allow(clippy::needless_pass_by_value)]
        fn focusable_with_requester(
            requester: #crate_path::FocusRequester,
        ) -> #crate_path::FocusNode {
            focus_requester_with(requester);
            focus_target()
        }

        #[allow(clippy::needless_pass_by_value)]
        fn focus_scope_with(scope: #crate_path::FocusScopeNode) -> #crate_path::FocusScopeNode {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime.register_current_focus_scope(scope);
            });
            scope
        }

        fn focus_scope() -> #crate_path::FocusScopeNode {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                if let Some(scope) = runtime.current_focus_scope_handle() {
                    scope
                } else {
                    let scope = focus_scope_handle();
                    runtime.ensure_current_focus_scope(scope);
                    scope
                }
            })
        }

        #[allow(clippy::needless_pass_by_value)]
        fn focus_group_with(group: #crate_path::FocusGroupNode) -> #crate_path::FocusGroupNode {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime.register_current_focus_group(group);
            });
            group
        }

        fn focus_group() -> #crate_path::FocusGroupNode {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                if let Some(group) = runtime.current_focus_group_handle() {
                    group
                } else {
                    let group = focus_group_handle();
                    runtime.ensure_current_focus_group(group);
                    group
                }
            })
        }

        fn focus_restorer() -> #crate_path::FocusScopeNode {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                if let Some(scope) = runtime.current_focus_scope_handle() {
                    scope
                } else {
                    let scope = focus_scope_handle();
                    runtime.ensure_current_focus_scope(scope);
                    scope
                }
            })
        }

        #[allow(clippy::needless_pass_by_value)]
        fn focus_restorer_with_fallback(
            fallback: #crate_path::FocusRequester,
        ) -> #crate_path::FocusScopeNode {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                let scope = if let Some(scope) = runtime.current_focus_scope_handle() {
                    scope
                } else {
                    let scope = focus_scope_handle();
                    runtime.ensure_current_focus_scope(scope);
                    scope
                };
                runtime.set_current_focus_restorer_fallback(fallback);
                scope
            })
        }

        #[allow(clippy::needless_pass_by_value)]
        fn focus_properties(properties: #crate_path::FocusProperties) {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime.set_current_focus_properties(properties);
            });
        }

        #[allow(clippy::needless_pass_by_value)]
        fn focus_traversal_policy(policy: #crate_path::FocusTraversalPolicy) {
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime.set_current_focus_traversal_policy(policy);
            });
        }

        #[allow(clippy::needless_pass_by_value)]
        fn on_focus_changed<F>(fun: F)
        where
            F: Into<#crate_path::CallbackWith<#crate_path::FocusState>>,
        {
            use #crate_path::runtime::TesseraRuntime;

            let handler: #crate_path::CallbackWith<#crate_path::FocusState> = fun.into();
            TesseraRuntime::with_mut(|runtime| {
                runtime.set_current_focus_changed_handler(handler);
            });
        }

        #[allow(clippy::needless_pass_by_value)]
        fn on_focus_event<F>(fun: F)
        where
            F: Into<#crate_path::CallbackWith<#crate_path::FocusState>>,
        {
            use #crate_path::runtime::TesseraRuntime;

            let handler: #crate_path::CallbackWith<#crate_path::FocusState> = fun.into();
            TesseraRuntime::with_mut(|runtime| {
                runtime.set_current_focus_event_handler(handler);
            });
        }

        fn focus_manager() -> #crate_path::FocusManager {
            #crate_path::FocusManager::current()
        }
    }
}

/// Helper: tokens to inject `callback`, `callback_with`, and `render_slot`.
fn callback_helpers_inject_tokens(crate_path: &syn::Path) -> proc_macro2::TokenStream {
    quote! {
        #[allow(clippy::needless_pass_by_value)]
        fn remember_callback<F>(fun: F) -> #crate_path::Callback
        where
            F: Fn() + Send + Sync + 'static,
        {
            #crate_path::Callback::new(fun)
        }

        #[allow(clippy::needless_pass_by_value)]
        fn remember_callback_with<T, R, F>(fun: F) -> #crate_path::CallbackWith<T, R>
        where
            T: Send + Sync + 'static,
            R: Send + Sync + 'static,
            F: Fn(T) -> R + Send + Sync + 'static,
        {
            #crate_path::CallbackWith::new(fun)
        }

        #[allow(clippy::needless_pass_by_value)]
        fn remember_render_slot<F>(fun: F) -> #crate_path::RenderSlot
        where
            F: Fn() + Send + Sync + 'static,
        {
            #crate_path::RenderSlot::new(fun)
        }
    }
}

struct PropFieldSpec {
    ident: Ident,
    ty: Type,
    setter: SetterAttrConfig,
    helper: Option<PropHelperKind>,
    skip_eq: bool,
}

fn generate_default_setter_method(
    field: &PropFieldSpec,
) -> syn::Result<Option<proc_macro2::TokenStream>> {
    if field.setter.skip {
        return Ok(None);
    }

    let ident = &field.ident;
    let method_doc = format!("Set `{ident}`.");
    let field_ty = &field.ty;
    if let Some(inner_ty) = option_inner_type(field_ty) {
        let method = if field.setter.into {
            quote! {
                #[doc = #method_doc]
                pub fn #ident(mut self, #ident: impl Into<#inner_ty>) -> Self {
                    self.#ident = Some(#ident.into());
                    self
                }
            }
        } else {
            quote! {
                #[doc = #method_doc]
                pub fn #ident(mut self, #ident: #inner_ty) -> Self {
                    self.#ident = Some(#ident);
                    self
                }
            }
        };
        return Ok(Some(method));
    }

    let method = if field.setter.into {
        quote! {
            #[doc = #method_doc]
            pub fn #ident(mut self, #ident: impl Into<#field_ty>) -> Self {
                self.#ident = #ident.into();
                self
            }
        }
    } else {
        quote! {
            #[doc = #method_doc]
            pub fn #ident(mut self, #ident: #field_ty) -> Self {
                self.#ident = #ident;
                self
            }
        }
    };
    Ok(Some(method))
}

fn generate_helper_setter_methods(
    field: &PropFieldSpec,
    helper: PropHelperKind,
    crate_path: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &field.ident;
    let shared_ident = format_ident!("{}_shared", ident);
    let helper_doc = format!("Set `{ident}` from a closure.");
    let shared_doc = format!("Set `{ident}` from a shared handle.");
    let (value_ty, wrap_some) = match option_inner_type(&field.ty) {
        Some(inner) => (inner, true),
        None => (field.ty.clone(), false),
    };

    match helper {
        PropHelperKind::Callback => {
            let matches_type = matches!(
                &value_ty,
                Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Callback")
            );
            if !matches_type {
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    "`#[prop(callback)]` requires `Callback` or `Option<Callback>`",
                ));
            }

            let closure_assign = if wrap_some {
                quote! { Some(#crate_path::Callback::new(#ident)) }
            } else {
                quote! { #crate_path::Callback::new(#ident) }
            };
            let shared_assign = if wrap_some {
                quote! { Some(#ident.into()) }
            } else {
                quote! { #ident.into() }
            };

            Ok(quote! {
                #[doc = #helper_doc]
                pub fn #ident<F>(mut self, #ident: F) -> Self
                where
                    F: Fn() + Send + Sync + 'static,
                {
                    self.#ident = #closure_assign;
                    self
                }

                #[doc = #shared_doc]
                pub fn #shared_ident(mut self, #ident: impl Into<#crate_path::Callback>) -> Self {
                    self.#ident = #shared_assign;
                    self
                }
            })
        }
        PropHelperKind::CallbackWith => {
            let Some((arg_ty, ret_ty)) = parse_functor_signature(&value_ty, "CallbackWith") else {
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    "`#[prop(callback_with)]` requires `CallbackWith<T, R>` or `Option<CallbackWith<T, R>>`",
                ));
            };

            let closure_assign = if wrap_some {
                quote! { Some(#crate_path::CallbackWith::new(#ident)) }
            } else {
                quote! { #crate_path::CallbackWith::new(#ident) }
            };
            let shared_assign = if wrap_some {
                quote! { Some(#ident.into()) }
            } else {
                quote! { #ident.into() }
            };

            Ok(quote! {
                #[doc = #helper_doc]
                pub fn #ident<F>(mut self, #ident: F) -> Self
                where
                    F: Fn(#arg_ty) -> #ret_ty + Send + Sync + 'static,
                {
                    self.#ident = #closure_assign;
                    self
                }

                #[doc = #shared_doc]
                pub fn #shared_ident(
                    mut self,
                    #ident: impl Into<#crate_path::CallbackWith<#arg_ty, #ret_ty>>,
                ) -> Self {
                    self.#ident = #shared_assign;
                    self
                }
            })
        }
        PropHelperKind::RenderSlot => {
            let matches_type = matches!(
                &value_ty,
                Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "RenderSlot")
            );
            if !matches_type {
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    "`#[prop(render_slot)]` requires `RenderSlot` or `Option<RenderSlot>`",
                ));
            }

            let closure_assign = if wrap_some {
                quote! { Some(#crate_path::RenderSlot::new(#ident)) }
            } else {
                quote! { #crate_path::RenderSlot::new(#ident) }
            };
            let shared_assign = if wrap_some {
                quote! { Some(#ident.into()) }
            } else {
                quote! { #ident.into() }
            };

            Ok(quote! {
                #[doc = #helper_doc]
                pub fn #ident<F>(mut self, #ident: F) -> Self
                where
                    F: Fn() + Send + Sync + 'static,
                {
                    self.#ident = #closure_assign;
                    self
                }

                #[doc = #shared_doc]
                pub fn #shared_ident(mut self, #ident: impl Into<#crate_path::RenderSlot>) -> Self {
                    self.#ident = #shared_assign;
                    self
                }
            })
        }
        PropHelperKind::RenderSlotWith => {
            let Some((arg_ty, ret_ty)) = parse_functor_signature(&value_ty, "RenderSlotWith")
            else {
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    "`#[prop(render_slot_with)]` requires `RenderSlotWith<T, R>` or `Option<RenderSlotWith<T, R>>`",
                ));
            };

            let closure_assign = if wrap_some {
                quote! { Some(#crate_path::RenderSlotWith::new(#ident)) }
            } else {
                quote! { #crate_path::RenderSlotWith::new(#ident) }
            };
            let shared_assign = if wrap_some {
                quote! { Some(#ident.into()) }
            } else {
                quote! { #ident.into() }
            };

            Ok(quote! {
                #[doc = #helper_doc]
                pub fn #ident<F>(mut self, #ident: F) -> Self
                where
                    F: Fn(#arg_ty) -> #ret_ty + Send + Sync + 'static,
                {
                    self.#ident = #closure_assign;
                    self
                }

                #[doc = #shared_doc]
                pub fn #shared_ident(
                    mut self,
                    #ident: impl Into<#crate_path::RenderSlotWith<#arg_ty, #ret_ty>>,
                ) -> Self {
                    self.#ident = #shared_assign;
                    self
                }
            })
        }
    }
}

/// Automatically converts a struct into component props and
/// generates convenient setter methods.
///
/// For common fields, it generates a single setter with the same field name.
/// For special field types such as `Callback`, `CallbackWith`, `RenderSlot`,
/// and `RenderSlotWith`, it additionally generates convenient setters that
/// accept closures and automatically wrap them into the corresponding helper
/// types.
///
/// For `Option<T>`, by default it tries to generate a setter that accepts `T`,
/// and wraps it into `Some(T)` internally, unless you use
/// `#[prop(skip_setter)]` to skip generation and write a manual setter.
///
/// For fields where you do not want a generated setter, you can mark the field
/// with `#[prop(skip_setter)]` to prevent generating that setter method.
///
/// For fields that should not participate in prop comparison, you can mark them
/// with `#[prop(skip_eq)]` to avoid triggering component updates when they
/// change.
#[proc_macro_derive(Prop, attributes(prop))]
pub fn derive_prop(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let container_attr = match parse_prop_container_attr(&input.attrs) {
        Ok(config) => config,
        Err(err) => return err.to_compile_error().into(),
    };
    let container_setter_attr = match parse_setter_attr(&input.attrs) {
        Ok(config) => config,
        Err(err) => return err.to_compile_error().into(),
    };
    let crate_path = container_attr
        .crate_path
        .unwrap_or_else(|| syn::parse_quote!(::tessera_ui));

    let Data::Struct(data_struct) = &input.data else {
        return syn::Error::new_spanned(
            &name,
            "#[derive(Prop)] only supports structs with named fields",
        )
        .to_compile_error()
        .into();
    };
    let Fields::Named(fields) = &data_struct.fields else {
        return syn::Error::new_spanned(
            &name,
            "#[derive(Prop)] only supports structs with named fields",
        )
        .to_compile_error()
        .into();
    };

    let mut methods = Vec::new();
    let mut compare_fields = Vec::new();

    for field in &fields.named {
        let Some(ident) = &field.ident else {
            return syn::Error::new_spanned(
                field,
                "#[derive(Prop)] only supports named struct fields",
            )
            .to_compile_error()
            .into();
        };

        let field_setter_attr = match parse_setter_attr(&field.attrs) {
            Ok(config) => config,
            Err(err) => return err.to_compile_error().into(),
        };
        let prop_attr = match parse_prop_field_attr(&field.attrs) {
            Ok(config) => config,
            Err(err) => return err.to_compile_error().into(),
        };
        let setter = merge_setter_attr(container_setter_attr, field_setter_attr);
        let helper = infer_prop_helper_kind(&field.ty);

        let spec = PropFieldSpec {
            ident: ident.clone(),
            ty: field.ty.clone(),
            setter,
            helper,
            skip_eq: prop_attr.skip_eq,
        };

        let wants_setter = !setter.skip;
        if wants_setter {
            if let Some(helper) = spec.helper {
                match generate_helper_setter_methods(&spec, helper, &crate_path) {
                    Ok(method_block) => methods.push(method_block),
                    Err(err) => return err.to_compile_error().into(),
                }
            } else {
                match generate_default_setter_method(&spec) {
                    Ok(Some(method)) => methods.push(method),
                    Ok(None) => {}
                    Err(err) => return err.to_compile_error().into(),
                }
            }
        }

        if !spec.skip_eq {
            let field_ident = &spec.ident;
            compare_fields.push(field_compare_expr(field_ident, &spec.ty));
        }
    }

    let prop_eq_expr = if compare_fields.is_empty() {
        quote! { true }
    } else {
        quote! { true #(&& #compare_fields)* }
    };

    let partial_eq_impl = quote! {
        impl #impl_generics ::core::cmp::PartialEq for #name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                #prop_eq_expr
            }
        }
    };

    let prop_impl = quote! {
        impl #impl_generics #crate_path::Prop for #name #ty_generics #where_clause {
            fn prop_eq(&self, other: &Self) -> bool {
                <Self as ::core::cmp::PartialEq>::eq(self, other)
            }
        }
    };

    let setters_impl = if methods.is_empty() {
        quote! {}
    } else {
        quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                #(#methods)*
            }
        }
    };

    quote! {
        #partial_eq_impl
        #prop_impl
        #setters_impl
    }
    .into()
}

/// Helper: tokens to compute a stable component type id based on module path +
/// function name.
fn component_type_id_tokens(fn_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            module_path!().hash(&mut hasher);
            stringify!(#fn_name).hash(&mut hasher);
            hasher.finish()
        }
    }
}

struct ControlFlowInstrumenter {
    /// counter to generate unique IDs in current function
    counter: usize,
    /// seed to prevent ID collisions across functions
    seed: u64,
}

impl ControlFlowInstrumenter {
    fn new(seed: u64) -> Self {
        Self { counter: 0, seed }
    }

    /// Generate the next unique group ID
    fn next_group_id(&mut self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.seed.hash(&mut hasher);
        self.counter.hash(&mut hasher);
        self.counter += 1;
        hasher.finish()
    }

    /// Wrap an expression in a GroupGuard block
    ///
    /// Before transform: expr
    /// After transform: { let _group_guard =
    /// ::tessera_ui::runtime::GroupGuard::new(#id); expr }
    fn wrap_expr_in_group(&mut self, expr: &mut Expr) {
        // Recursively visit sub-expressions (depth-first) to ensure nested structures
        // are wrapped
        self.visit_expr_mut(expr);
        let group_id = self.next_group_id();
        // Use fully-qualified path ::tessera_ui to avoid relying on a crate alias
        let original_expr = &expr;
        let new_expr: Expr = parse_quote! {
            {
                let _group_guard = ::tessera_ui::runtime::GroupGuard::new(#group_id);
                #original_expr
            }
        };
        *expr = new_expr;
    }

    /// Wrap a block in a GroupGuard block
    fn wrap_block_in_group(&mut self, block: &mut Block) {
        // Recursively instrument nested expressions before wrapping the block
        self.visit_block_mut(block);

        let group_id = self.next_group_id();
        let original_stmts = &block.stmts;

        let new_block: Block = parse_quote! {
            {
                let _group_guard = ::tessera_ui::runtime::GroupGuard::new(#group_id);
                #(#original_stmts)*
            }
        };

        *block = new_block;
    }

    /// Wrap a block in a path-only group block.
    fn wrap_block_in_path_group(&mut self, block: &mut Block) {
        self.visit_block_mut(block);

        let group_id = self.next_group_id();
        let original_stmts = &block.stmts;

        let new_block: Block = parse_quote! {
            {
                let _group_guard = ::tessera_ui::runtime::PathGroupGuard::new(#group_id);
                #(#original_stmts)*
            }
        };

        *block = new_block;
    }
}

impl VisitMut for ControlFlowInstrumenter {
    fn visit_expr_if_mut(&mut self, i: &mut syn::ExprIf) {
        self.visit_expr_mut(&mut i.cond);
        self.wrap_block_in_group(&mut i.then_branch);
        if let Some((_, else_branch)) = &mut i.else_branch {
            match &mut **else_branch {
                Expr::Block(block_expr) => {
                    self.wrap_block_in_group(&mut block_expr.block);
                }
                Expr::If(_) => {
                    self.visit_expr_mut(else_branch);
                }
                _ => {
                    self.wrap_expr_in_group(else_branch);
                }
            }
        }
    }

    fn visit_expr_match_mut(&mut self, m: &mut syn::ExprMatch) {
        self.visit_expr_mut(&mut m.expr);
        for arm in &mut m.arms {
            self.wrap_expr_in_group(&mut arm.body);
        }
    }

    fn visit_expr_for_loop_mut(&mut self, f: &mut syn::ExprForLoop) {
        self.visit_expr_mut(&mut f.expr);
        self.wrap_block_in_path_group(&mut f.body);
    }

    fn visit_expr_while_mut(&mut self, w: &mut syn::ExprWhile) {
        self.visit_expr_mut(&mut w.cond);
        self.wrap_block_in_path_group(&mut w.body);
    }

    fn visit_expr_loop_mut(&mut self, l: &mut syn::ExprLoop) {
        self.wrap_block_in_path_group(&mut l.body);
    }
}

/// Transforms a regular Rust function into a Tessera UI component.
///
/// # Usage
///
/// Annotate a free function (no captured self) with `#[tessera]`. You may then
/// (optionally) call any of the injected helpers exactly once (last call wins
/// if repeated).
///
/// # Parameters
///
/// * Attribute arguments are currently unused; pass nothing or `#[tessera]`.
///
/// # When NOT to Use
///
/// * For function that should not be a ui component.
///
/// # See Also
///
/// * [`#[shard]`](crate::shard) for navigation‑aware components with injectable
///   shard state.
#[proc_macro_attribute]
pub fn tessera(attr: TokenStream, item: TokenStream) -> TokenStream {
    let crate_path: syn::Path = match parse_crate_path(attr) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error().into(),
    };

    // Parse the input function that will be transformed into a component
    let mut input_fn = parse_macro_input!(item as ItemFn);
    if let Some(conflicting_attr) = input_fn
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("shard"))
    {
        return syn::Error::new_spanned(
            conflicting_attr,
            "#[tessera] and #[shard] cannot be combined on the same function; use #[shard] only",
        )
        .to_compile_error()
        .into();
    }
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_sig = &input_fn.sig;
    let prop_signature = match strict_prop_signature(fn_sig) {
        Ok(v) => v,
        Err(err) => return err.to_compile_error().into(),
    };
    let prop_type = prop_signature.prop_type();

    // Generate a stable hash seed based on function name in order to avoid ID
    // collisions
    let mut hasher = DefaultHasher::new();
    input_fn.sig.ident.to_string().hash(&mut hasher);
    let seed = hasher.finish();

    // Modify the function body to instrument control flow with GroupGuard
    let mut instrumenter = ControlFlowInstrumenter::new(seed);
    instrumenter.visit_block_mut(&mut input_fn.block);
    let fn_block = &input_fn.block;

    // Prepare token fragments using helpers to keep function small and readable
    let register_tokens = register_node_tokens(&crate_path, fn_name);
    let layout_tokens = layout_inject_tokens(&crate_path);
    let state_tokens = input_handler_inject_tokens(&crate_path);
    let callback_helper_tokens = callback_helpers_inject_tokens(&crate_path);
    let replay_tokens = replay_register_tokens(&crate_path, fn_name, &prop_signature);
    let prop_assert_tokens = prop_assert_tokens(&crate_path, &prop_type);
    let component_type_id_tokens = component_type_id_tokens(fn_name);

    // Generate the transformed function with Tessera runtime integration
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            let __tessera_component_type_id: u64 = #component_type_id_tokens;
            let __tessera_phase_guard = {
                use #crate_path::runtime::{RuntimePhase, push_phase};
                push_phase(RuntimePhase::Build)
            };
            let __tessera_fn_name: &str = stringify!(#fn_name);
            let __tessera_node_id = #register_tokens;

            // Inject guard to pop component node on function exit
            let _component_scope_guard = {
                struct ComponentScopeGuard;
                impl Drop for ComponentScopeGuard {
                    fn drop(&mut self) {
                        use #crate_path::runtime::TesseraRuntime;
                        TesseraRuntime::with_mut(|runtime| {
                            runtime.finalize_current_layout_spec_dirty();
                            runtime.component_tree.pop_node();
                        });
                    }
                }
                ComponentScopeGuard
            };

            // Track current node for control-flow instrumentation
            let _node_ctx_guard = {
                use #crate_path::runtime::push_current_node;
                push_current_node(
                    __tessera_node_id,
                    __tessera_component_type_id,
                    __tessera_fn_name,
                )
            };

            let __tessera_instance_key: u64 = #crate_path::runtime::current_instance_key();
            let __tessera_instance_logic_id: u64 =
                #crate_path::runtime::current_instance_logic_id();
            let _instance_ctx_guard = {
                use #crate_path::runtime::push_current_component_instance_key;
                push_current_component_instance_key(__tessera_instance_key)
            };
            {
                use #crate_path::runtime::TesseraRuntime;
                TesseraRuntime::with_mut(|runtime| {
                    runtime.set_current_node_identity(
                        __tessera_instance_key,
                        __tessera_instance_logic_id,
                    );
                });
            }
            #prop_assert_tokens
            #crate_path::context::record_current_context_snapshot_for(__tessera_instance_key);
            #replay_tokens
            if __tessera_component_reused {
                return;
            }
            // Inject helper tokens
            #layout_tokens
            #state_tokens
            #callback_helper_tokens
            // Execute user's function body
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// Generates platform-specific entry points from a shared `run` function.
///
/// # Usage
///
/// Annotate a public zero-argument function that returns [`EntryPoint`].
#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let crate_path: syn::Path = match parse_crate_path(attr) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error().into(),
    };
    let mut input_fn = parse_macro_input!(item as ItemFn);

    if !input_fn.sig.inputs.is_empty() {
        return syn::Error::new_spanned(
            &input_fn.sig.inputs,
            "entry functions must not accept arguments",
        )
        .to_compile_error()
        .into();
    }

    input_fn.attrs.retain(|attr| !attr.path().is_ident("entry"));
    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        #[cfg(target_os = "android")]
        #[unsafe(no_mangle)]
        fn android_main(android_app: #crate_path::winit::platform::android::activity::AndroidApp) {
            if let Err(err) = #fn_name().run_android(android_app) {
                eprintln!("App failed to run: {err}");
            }
        }
    };

    expanded.into()
}

/// Transforms a function into a *shard component* that can be navigated to via
/// the routing system and (optionally) provided with a lazily‑initialized
/// per‑shard state.
///
/// # Features
///
/// * Generates a `StructNameDestination` (UpperCamelCase + `Destination`)
///   implementing `tessera_shard::router::RouterDestination`
/// * Optional state injection via `#[shard(state = T)]`, where `T`:
///   - Must implement `Default + Send + Sync + 'static`
///   - Is constructed (or reused) and exposed as local variable `state` with
///     type `tessera_shard::ShardState<T>`
/// * Produces a stable shard ID: `module_path!()::function_name`
///
/// # Lifecycle
///
/// Controlled by the `lifecycle` shard attribute argument.
/// * Default: `Shard` – state is removed when the destination is `pop()`‑ed
/// * Override: `#[shard(lifecycle = scope)]` to persist for the lifetime of
///   current `router_scope`
///
/// Route-scoped state is removed on route pop/clear. Scope-scoped state is
/// removed when the router scope is dropped.
///
/// # Parameter Transformation
///
/// * Function parameters are treated as explicit destination props.
/// * When `state = T` is configured, shard state is injected as local variable
///   `state` and does not appear in the function signature.
///
/// # Generated Destination (Conceptual)
///
/// ```rust,ignore
/// struct ProfilePageDestination { /* non-state params as public fields */ }
/// impl RouterDestination for ProfilePageDestination {
///     fn exec_component(&self) { profile_page(/* fields */); }
///     fn shard_id(&self) -> &'static str { "<module>::profile_page" }
/// }
/// ```
///
/// # Limitations
///
/// * Do not manually implement `RouterDestination` for these pages; rely on
///   generation
///
/// # See Also
///
/// * Routing helpers: `tessera_ui::router::{Router, router_root}`
/// * Scoped router internals: `tessera_shard::router::Router`
///
/// # Errors
///
/// Emits a compile error if unsupported `lifecycle` is provided, or if
/// `lifecycle` is used without `state`.
#[cfg(feature = "shard")]
#[proc_macro_attribute]
pub fn shard(attr: TokenStream, input: TokenStream) -> TokenStream {
    use heck::ToUpperCamelCase;
    let shard_args = if attr.is_empty() {
        ShardMacroArgs::default()
    } else {
        match syn::parse::<ShardMacroArgs>(attr) {
            Ok(value) => value,
            Err(err) => return err.to_compile_error().into(),
        }
    };

    let crate_path: syn::Path = shard_args
        .crate_path
        .unwrap_or_else(|| syn::parse_quote!(::tessera_ui));

    let func = parse_macro_input!(input as ItemFn);
    if let Some(conflicting_attr) = func
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("tessera"))
    {
        return syn::Error::new_spanned(
            conflicting_attr,
            "#[shard] already defines a component boundary; do not combine #[shard] with #[tessera]",
        )
        .to_compile_error()
        .into();
    }

    let shard_params = match parse_shard_params(&func.sig) {
        Ok(params) => params,
        Err(err) => return err.to_compile_error().into(),
    };

    let state_type = shard_args.state_type;
    let state_lifecycle_tokens = match shard_args.lifecycle {
        Some(lifecycle) => {
            let lifecycle_name = lifecycle.to_string().to_lowercase();
            if state_type.is_none() {
                return syn::Error::new_spanned(
                    lifecycle,
                    "`lifecycle` requires `state` in #[shard(...)]",
                )
                .to_compile_error()
                .into();
            }
            if lifecycle_name == "scope" {
                quote! { #crate_path::tessera_shard::ShardStateLifeCycle::Scope }
            } else if lifecycle_name == "shard" {
                quote! { #crate_path::tessera_shard::ShardStateLifeCycle::Shard }
            } else {
                return syn::Error::new_spanned(
                    lifecycle,
                    "unsupported `lifecycle` in #[shard(...)]: expected `scope` or `shard`",
                )
                .to_compile_error()
                .into();
            }
        }
        None => quote! { #crate_path::tessera_shard::ShardStateLifeCycle::Shard },
    };

    let func_body = func.block;
    let func_name_str = func.sig.ident.to_string();

    let func_attrs = &func.attrs;
    let func_vis = &func.vis;
    let func_sig_modified = &func.sig;

    // Generate struct name for the new RouterDestination
    let func_name = func.sig.ident.clone();
    let struct_name = syn::Ident::new(
        &format!("{}Destination", func_name_str.to_upper_camel_case()),
        func_name.span(),
    );
    let inner_component_name = format_ident!("__{}_shard_component", func_name);
    let shard_props_name = syn::Ident::new(
        &format!("{}ShardProps", func_name_str.to_upper_camel_case()),
        func_name.span(),
    );

    // Generate fields for the new struct that will implement `RouterDestination`
    let dest_fields = shard_params.iter().map(|param| {
        let ident = &param.ident;
        let ty = &param.ty;
        quote! { pub #ident: #ty }
    });

    // Keep all explicit function parameters as destination props.
    let param_idents: Vec<_> = shard_params
        .iter()
        .map(|param| param.ident.clone())
        .collect();

    let shard_prop_fields = shard_params.iter().map(|param| {
        let ident = &param.ident;
        let ty = &param.ty;
        quote! { #ident: #ty }
    });

    let expanded = {
        // `exec_component` only passes destination prop fields.
        let exec_args = param_idents
            .iter()
            .map(|ident| quote! { self.#ident.clone() });
        let shard_prop_init = param_idents.iter().map(|ident| quote! { #ident });
        let shard_prop_bindings = param_idents.iter().map(|ident| {
            quote! {
                let #ident = __tessera_shard_props.#ident.clone();
            }
        });

        if let Some(state_type) = state_type {
            quote! {
                #func_vis struct #struct_name {
                    #(#dest_fields),*
                }

                impl #crate_path::tessera_shard::router::RouterDestination for #struct_name {
                    fn exec_component(&self) {
                        #func_name(
                            #(
                                #exec_args
                            ),*
                        );
                    }

                    fn shard_id(&self) -> &'static str {
                        concat!(module_path!(), "::", #func_name_str)
                    }
                }

                #(#func_attrs)*
                #func_vis #func_sig_modified {
                    #[derive(Clone)]
                    struct #shard_props_name {
                        #(#shard_prop_fields),*
                    }

                    impl #crate_path::Prop for #shard_props_name {
                        fn prop_eq(&self, _other: &Self) -> bool {
                            false
                        }
                    }

                    #[#crate_path::tessera(#crate_path)]
                    fn #inner_component_name(__tessera_shard_props: &#shard_props_name) {
                        #(#shard_prop_bindings)*

                        const SHARD_ID: &str = concat!(module_path!(), "::", #func_name_str);
                        #crate_path::router::with_current_router_shard_state::<#state_type, _, _>(
                            SHARD_ID,
                            #state_lifecycle_tokens,
                            |state| {
                                #func_body
                            },
                        )
                    }

                    let __tessera_shard_props = #shard_props_name {
                        #(#shard_prop_init),*
                    };
                    #inner_component_name(&__tessera_shard_props);
                }
            }
        } else {
            quote! {
                #func_vis struct #struct_name {
                    #(#dest_fields),*
                }

                impl #crate_path::tessera_shard::router::RouterDestination for #struct_name {
                    fn exec_component(&self) {
                        #func_name(
                            #(
                                #exec_args
                            ),*
                        );
                    }

                    fn shard_id(&self) -> &'static str {
                        concat!(module_path!(), "::", #func_name_str)
                    }
                }

                #(#func_attrs)*
                #func_vis #func_sig_modified {
                    #[derive(Clone)]
                    struct #shard_props_name {
                        #(#shard_prop_fields),*
                    }

                    impl #crate_path::Prop for #shard_props_name {
                        fn prop_eq(&self, _other: &Self) -> bool {
                            false
                        }
                    }

                    #[#crate_path::tessera(#crate_path)]
                    fn #inner_component_name(__tessera_shard_props: &#shard_props_name) {
                        #(#shard_prop_bindings)*
                        #func_body
                    }

                    let __tessera_shard_props = #shard_props_name {
                        #(#shard_prop_init),*
                    };
                    #inner_component_name(&__tessera_shard_props);
                }
            }
        }
    };

    TokenStream::from(expanded)
}
