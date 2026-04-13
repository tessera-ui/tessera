//! # Tessera Macros
//!
//! This crate provides procedural macros for the Tessera UI framework.
//! The main export is the `#[tessera]` attribute macro, which transforms
//! regular Rust functions into Tessera UI components.

use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Block, Expr, FnArg, GenericArgument, Ident, ItemFn, Pat, Path, PathArguments, Token, Type,
    parse::Parse, parse_macro_input, parse_quote, visit_mut::VisitMut,
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

fn is_unit_type(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(tuple) if tuple.elems.is_empty())
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
    shard_crate_path: Option<Path>,
    state_type: Option<Type>,
    lifecycle: Option<Ident>,
}

#[cfg(feature = "shard")]
struct ShardParam {
    ident: Ident,
    ty: Type,
    is_router: bool,
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
                let is_router = pat_type
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("router"));
                params.push(ShardParam {
                    ident: pat_ident.ident.clone(),
                    ty: (*pat_type.ty).clone(),
                    is_router,
                });
            }
        }
    }
    Ok(params)
}

#[cfg(feature = "shard")]
fn is_state_router_controller_type(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    let Some(state_segment) = type_path.path.segments.last() else {
        return false;
    };
    if state_segment.ident != "State" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &state_segment.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(Type::Path(controller_path))) = args.args.first() else {
        return false;
    };
    controller_path
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "RouterController")
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
                "shard_crate_path" => {
                    if args.shard_crate_path.is_some() {
                        return Err(syn::Error::new(
                            key.span(),
                            "duplicate `shard_crate_path` argument",
                        ));
                    }
                    args.shard_crate_path = Some(input.parse::<Path>()?);
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
                        "unsupported #[shard(...)] argument; expected `state`, `lifecycle`, `crate_path`, or `shard_crate_path`",
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
        #crate_path::__private::register_component_node(
            stringify!(#fn_name),
            __tessera_component_type_id,
        )
    }
}

/// Parse and validate strict component props signature:
/// `fn component()` or `fn component(foo: T, bar: U)`.
enum ComponentPropSignature {
    Unit,
    Params(Vec<PropFieldSpec>),
}

fn strict_prop_signature(sig: &syn::Signature) -> Result<ComponentPropSignature, syn::Error> {
    if sig.inputs.is_empty() {
        return Ok(ComponentPropSignature::Unit);
    }

    let mut fields = Vec::new();
    for arg in &sig.inputs {
        let FnArg::Typed(arg) = arg else {
            return Err(syn::Error::new_spanned(
                arg,
                "#[tessera] methods are not supported; use free functions with named parameters",
            ));
        };

        let Pat::Ident(pat_ident) = arg.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &arg.pat,
                "component parameters must be named identifiers",
            ));
        };

        if pat_ident.by_ref.is_some() || pat_ident.mutability.is_some() {
            return Err(syn::Error::new_spanned(
                pat_ident,
                "component parameters must be plain named bindings like `foo: T`",
            ));
        }

        if matches!(arg.ty.as_ref(), Type::Reference(_)) {
            return Err(syn::Error::new_spanned(
                &arg.ty,
                "component parameters must be owned types; borrowed parameter types are not supported",
            ));
        }

        let field_setter_attr = parse_setter_attr(&arg.attrs)?;
        let prop_attr = parse_prop_field_attr(&arg.attrs)?;
        let default_expr = parse_component_default_attr(&arg.attrs)?;
        fields.push(PropFieldSpec {
            ident: pat_ident.ident.clone(),
            ty: (*arg.ty).clone(),
            setter: field_setter_attr,
            helper: infer_prop_helper_kind(arg.ty.as_ref()),
            skip_eq: prop_attr.skip_eq,
            default_expr,
        });
    }

    Ok(ComponentPropSignature::Params(fields))
}

/// Helper: tokens to attach replay metadata to the current component node.
fn replay_register_tokens(
    crate_path: &syn::Path,
    runner_name: &syn::Ident,
    prop_type: &syn::Type,
    props_expr: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        let __tessera_component_reused = {
            let __tessera_runner =
                #crate_path::__private::make_component_runner::<#prop_type>(#runner_name);
            #crate_path::__private::set_current_component_replay(__tessera_runner, #props_expr)
        };
    }
}

/// Helper: compile-time assertion that component props implement `Prop`.
fn prop_assert_tokens(crate_path: &syn::Path, prop_type: &syn::Type) -> proc_macro2::TokenStream {
    quote! {
        {
            fn __tessera_assert_prop<T: #crate_path::__private::Prop>() {}
            __tessera_assert_prop::<#prop_type>();
        }
    }
}

struct PropFieldSpec {
    ident: Ident,
    ty: Type,
    setter: SetterAttrConfig,
    helper: Option<PropHelperKind>,
    skip_eq: bool,
    default_expr: Option<Expr>,
}

fn parse_component_default_attr(attrs: &[syn::Attribute]) -> syn::Result<Option<Expr>> {
    let mut default_expr = None;
    for attr in attrs {
        if !attr.path().is_ident("default") {
            continue;
        }

        if default_expr.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "duplicate #[default(...)] on component parameter",
            ));
        }

        default_expr = Some(attr.parse_args::<Expr>()?);
    }

    Ok(default_expr)
}

fn generate_default_setter_method_for_path(
    field: &PropFieldSpec,
    field_path: &proc_macro2::TokenStream,
    set_flag_path: Option<&proc_macro2::TokenStream>,
) -> syn::Result<Option<proc_macro2::TokenStream>> {
    if field.setter.skip {
        return Ok(None);
    }

    let ident = &field.ident;
    let method_doc = format!("Set `{ident}`.");
    let field_ty = &field.ty;
    let set_flag_stmt = set_flag_path.map(|path| quote!(self.#path = true;));
    if let Some(inner_ty) = option_inner_type(field_ty) {
        let method = if field.setter.into {
            quote! {
                #[doc = #method_doc]
                pub fn #ident(mut self, #ident: impl Into<#inner_ty>) -> Self {
                    self.#field_path = Some(#ident.into());
                    #set_flag_stmt
                    self
                }
            }
        } else {
            quote! {
                #[doc = #method_doc]
                pub fn #ident(mut self, #ident: #inner_ty) -> Self {
                    self.#field_path = Some(#ident);
                    #set_flag_stmt
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
                self.#field_path = #ident.into();
                #set_flag_stmt
                self
            }
        }
    } else {
        quote! {
            #[doc = #method_doc]
            pub fn #ident(mut self, #ident: #field_ty) -> Self {
                self.#field_path = #ident;
                #set_flag_stmt
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
    field_path: &proc_macro2::TokenStream,
    set_flag_path: Option<&proc_macro2::TokenStream>,
) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &field.ident;
    let shared_ident = format_ident!("{}_shared", ident);
    let helper_doc = format!("Set `{ident}` from a closure.");
    let shared_doc = format!("Set `{ident}` from a shared handle.");
    let (value_ty, wrap_some) = match option_inner_type(&field.ty) {
        Some(inner) => (inner, true),
        None => (field.ty.clone(), false),
    };
    let set_flag_stmt = set_flag_path.map(|path| quote!(self.#path = true;));

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
                    self.#field_path = #closure_assign;
                    #set_flag_stmt
                    self
                }

                #[doc = #shared_doc]
                pub fn #shared_ident(mut self, #ident: impl Into<#crate_path::Callback>) -> Self {
                    self.#field_path = #shared_assign;
                    #set_flag_stmt
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

            let closure_assign = if is_unit_type(&arg_ty) {
                if wrap_some {
                    quote! { Some(#crate_path::CallbackWith::new(move |()| #ident())) }
                } else {
                    quote! { #crate_path::CallbackWith::new(move |()| #ident()) }
                }
            } else if wrap_some {
                quote! { Some(#crate_path::CallbackWith::new(#ident)) }
            } else {
                quote! { #crate_path::CallbackWith::new(#ident) }
            };
            let shared_assign = if wrap_some {
                quote! { Some(#ident.into()) }
            } else {
                quote! { #ident.into() }
            };

            let callback_bound = if is_unit_type(&arg_ty) {
                quote! { F: Fn() -> #ret_ty + Send + Sync + 'static }
            } else {
                quote! { F: Fn(#arg_ty) -> #ret_ty + Send + Sync + 'static }
            };

            Ok(quote! {
                #[doc = #helper_doc]
                pub fn #ident<F>(mut self, #ident: F) -> Self
                where
                    #callback_bound,
                {
                    self.#field_path = #closure_assign;
                    #set_flag_stmt
                    self
                }

                #[doc = #shared_doc]
                pub fn #shared_ident(
                    mut self,
                    #ident: impl Into<#crate_path::CallbackWith<#arg_ty, #ret_ty>>,
                ) -> Self {
                    self.#field_path = #shared_assign;
                    #set_flag_stmt
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
                    self.#field_path = #closure_assign;
                    #set_flag_stmt
                    self
                }

                #[doc = #shared_doc]
                pub fn #shared_ident(mut self, #ident: impl Into<#crate_path::RenderSlot>) -> Self {
                    self.#field_path = #shared_assign;
                    #set_flag_stmt
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

            let closure_assign = if is_unit_type(&arg_ty) {
                if wrap_some {
                    quote! { Some(#crate_path::RenderSlotWith::new(move |()| #ident())) }
                } else {
                    quote! { #crate_path::RenderSlotWith::new(move |()| #ident()) }
                }
            } else if wrap_some {
                quote! { Some(#crate_path::RenderSlotWith::new(#ident)) }
            } else {
                quote! { #crate_path::RenderSlotWith::new(#ident) }
            };
            let shared_assign = if wrap_some {
                quote! { Some(#ident.into()) }
            } else {
                quote! { #ident.into() }
            };

            let callback_bound = if is_unit_type(&arg_ty) {
                quote! { F: Fn() -> #ret_ty + Send + Sync + 'static }
            } else {
                quote! { F: Fn(#arg_ty) -> #ret_ty + Send + Sync + 'static }
            };

            Ok(quote! {
                #[doc = #helper_doc]
                pub fn #ident<F>(mut self, #ident: F) -> Self
                where
                    #callback_bound,
                {
                    self.#field_path = #closure_assign;
                    #set_flag_stmt
                    self
                }

                #[doc = #shared_doc]
                pub fn #shared_ident(
                    mut self,
                    #ident: impl Into<#crate_path::RenderSlotWith<#arg_ty, #ret_ty>>,
                ) -> Self {
                    self.#field_path = #shared_assign;
                    #set_flag_stmt
                    self
                }
            })
        }
    }
}

/// Automatically converts a struct into component props and
fn pascal_case_ident(base: &Ident, suffix: &str) -> Ident {
    let mut output = String::new();
    for segment in base
        .to_string()
        .split('_')
        .filter(|segment| !segment.is_empty())
    {
        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            output.extend(first.to_uppercase());
            output.push_str(chars.as_str());
        }
    }
    output.push_str(suffix);
    format_ident!("{output}")
}

fn hidden_props_ident(fn_name: &Ident) -> Ident {
    format_ident!("__Tessera{}Props", pascal_case_ident(fn_name, ""))
}

fn builder_ident(fn_name: &Ident) -> Ident {
    pascal_case_ident(fn_name, "Builder")
}

fn hidden_component_impl_ident(fn_name: &Ident) -> Ident {
    format_ident!("__tessera_{}_impl", fn_name)
}

fn generate_prop_like_impls(
    type_name: &Ident,
    fields: &[PropFieldSpec],
    crate_path: &Path,
) -> proc_macro2::TokenStream {
    let compare_fields: Vec<_> = fields
        .iter()
        .filter(|field| !field.skip_eq)
        .map(|field| field_compare_expr(&field.ident, &field.ty))
        .collect();
    let prop_eq_expr = if compare_fields.is_empty() {
        quote! { true }
    } else {
        quote! { true #(&& #compare_fields)* }
    };

    quote! {
        impl ::core::cmp::PartialEq for #type_name {
            fn eq(&self, other: &Self) -> bool {
                #prop_eq_expr
            }
        }

        impl #crate_path::__private::Prop for #type_name {
            fn prop_eq(&self, other: &Self) -> bool {
                <Self as ::core::cmp::PartialEq>::eq(self, other)
            }
        }
    }
}

fn generate_builder_methods(
    fields: &[PropFieldSpec],
    crate_path: &Path,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut methods = Vec::new();
    for field in fields {
        let ident = &field.ident;
        let field_path = quote!(props.#ident);
        let set_flag_ident = format_ident!("__tessera_set_{}", ident);
        let set_flag_path = quote!(props.#set_flag_ident);
        if let Some(helper) = field.helper {
            methods.push(generate_helper_setter_methods(
                field,
                helper,
                crate_path,
                &field_path,
                Some(&set_flag_path),
            )?);
        } else if let Some(method) =
            generate_default_setter_method_for_path(field, &field_path, Some(&set_flag_path))?
        {
            methods.push(method);
        }
    }
    Ok(methods)
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
    /// ::tessera_ui::__private::GroupGuard::new(#id); expr }
    fn wrap_expr_in_group(&mut self, expr: &mut Expr) {
        // Recursively visit sub-expressions (depth-first) to ensure nested structures
        // are wrapped
        self.visit_expr_mut(expr);
        let group_id = self.next_group_id();
        // Use fully-qualified path ::tessera_ui to avoid relying on a crate alias
        let original_expr = &expr;
        let new_expr: Expr = parse_quote! {
            {
                let _group_guard = ::tessera_ui::__private::GroupGuard::new(#group_id);
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
                let _group_guard = ::tessera_ui::__private::GroupGuard::new(#group_id);
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
                let _group_guard = ::tessera_ui::__private::PathGroupGuard::new(#group_id);
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
/// Annotate a plain free function with `#[tessera]`.
///
/// The macro turns the function into a Tessera component entrypoint. Public
/// components use the generated builder syntax, while the original function
/// body runs inside Tessera's build/replay context.
///
/// # Parameters
///
/// * Attribute arguments select the Tessera crate path. Use `#[tessera]` for
///   normal external authoring, or `#[tessera(crate)]` inside Tessera crates.
///
/// # When NOT to Use
///
/// * For functions that should not participate in the component tree.
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

    if input_fn.sig.constness.is_some()
        || input_fn.sig.asyncness.is_some()
        || input_fn.sig.unsafety.is_some()
        || input_fn.sig.abi.is_some()
        || input_fn.sig.variadic.is_some()
        || !input_fn.sig.generics.params.is_empty()
        || input_fn.sig.generics.where_clause.is_some()
    {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "#[tessera] components must be plain, non-generic free functions",
        )
        .to_compile_error()
        .into();
    }
    if !matches!(input_fn.sig.output, syn::ReturnType::Default) {
        return syn::Error::new_spanned(
            &input_fn.sig.output,
            "#[tessera] components must not return a value",
        )
        .to_compile_error()
        .into();
    }

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let prop_signature = match strict_prop_signature(&input_fn.sig) {
        Ok(v) => v,
        Err(err) => return err.to_compile_error().into(),
    };

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
    let component_type_id_tokens = component_type_id_tokens(fn_name);
    let expanded = match &prop_signature {
        ComponentPropSignature::Unit => {
            let fn_sig = &input_fn.sig;
            let prop_assert_tokens = prop_assert_tokens(&crate_path, &syn::parse_quote!(()));
            let unit_runner_ident = format_ident!("__tessera_{}_unit_runner", fn_name);
            let replay_tokens = replay_register_tokens(
                &crate_path,
                &unit_runner_ident,
                &syn::parse_quote!(()),
                quote!(&__tessera_unit_props),
            );

            quote! {
                fn #unit_runner_ident(_props: &()) {
                    #fn_name();
                }

                #(#fn_attrs)*
                #fn_vis #fn_sig {
                    let __tessera_component_type_id: u64 = #component_type_id_tokens;
                    let __tessera_phase_guard = {
                        use #crate_path::__private::{RuntimePhase, push_phase};
                        push_phase(RuntimePhase::Build)
                    };
                    let __tessera_fn_name: &str = stringify!(#fn_name);
                    let __tessera_node_id = #register_tokens;

                    let _node_ctx_guard = {
                        use #crate_path::__private::push_current_node;
                        push_current_node(
                            __tessera_node_id,
                            __tessera_component_type_id,
                            __tessera_fn_name,
                        )
                    };

                    let __tessera_instance_key: u64 = #crate_path::__private::current_instance_key();
                    let __tessera_instance_logic_id: u64 =
                        #crate_path::__private::current_instance_logic_id();
                    let _instance_ctx_guard = {
                        use #crate_path::__private::push_current_component_instance_key;
                        push_current_component_instance_key(__tessera_instance_key)
                    };
                    let _component_scope_guard = {
                        struct ComponentScopeGuard;
                        impl Drop for ComponentScopeGuard {
                            fn drop(&mut self) {
                                #crate_path::__private::finish_component_node();
                            }
                        }
                        ComponentScopeGuard
                    };
                    #crate_path::__private::set_current_node_identity(
                        __tessera_instance_key,
                        __tessera_instance_logic_id,
                    );
                    #prop_assert_tokens
                    #crate_path::__private::record_current_context_snapshot_for(__tessera_instance_key);
                    let __tessera_unit_props = ();
                    #replay_tokens
                    if __tessera_component_reused {
                        return;
                    }
                    #fn_block
                }
            }
        }
        ComponentPropSignature::Params(fields) => {
            let props_ident = hidden_props_ident(fn_name);
            let builder_ident = builder_ident(fn_name);
            let impl_ident = hidden_component_impl_ident(fn_name);
            let set_flag_idents: Vec<_> = fields
                .iter()
                .map(|field| format_ident!("__tessera_set_{}", field.ident))
                .collect();
            let field_defs: Vec<_> = fields
                .iter()
                .map(|field| {
                    let ident = &field.ident;
                    let ty = &field.ty;
                    quote!(#ident: #ty)
                })
                .collect();
            let set_flag_defs: Vec<_> = set_flag_idents
                .iter()
                .map(|ident| quote!(#ident: bool))
                .collect();
            let field_resolutions: Vec<_> = fields
                .iter()
                .zip(set_flag_idents.iter())
                .map(|(field, set_flag_ident)| {
                    let ident = &field.ident;
                    let ty = &field.ty;
                    if let Some(default_expr) = &field.default_expr {
                        quote! {
                            let #ident: #ty = if __tessera_props.#set_flag_ident {
                                __tessera_props.#ident.clone()
                            } else {
                                #default_expr
                            };
                        }
                    } else {
                        quote!(let #ident: #ty = __tessera_props.#ident.clone();)
                    }
                })
                .collect();
            let prop_impl_tokens = generate_prop_like_impls(&props_ident, fields, &crate_path);
            let builder_methods = match generate_builder_methods(fields, &crate_path) {
                Ok(methods) => methods,
                Err(err) => return err.to_compile_error().into(),
            };
            let prop_assert_tokens =
                prop_assert_tokens(&crate_path, &syn::parse_quote!(#props_ident));
            let replay_tokens = replay_register_tokens(
                &crate_path,
                &impl_ident,
                &syn::parse_quote!(#props_ident),
                quote!(__tessera_props),
            );

            quote! {
                #[derive(Clone, Default)]
                struct #props_ident {
                    #(#field_defs,)*
                    #(#set_flag_defs,)*
                }

                #prop_impl_tokens

                #[derive(Default)]
                #[doc = concat!("Builder returned by [`", stringify!(#fn_name), "`].")]
                #fn_vis struct #builder_ident {
                    props: #props_ident,
                }

                impl #builder_ident {
                    #(#builder_methods)*
                }

                impl Drop for #builder_ident {
                    fn drop(&mut self) {
                        #impl_ident(&self.props);
                    }
                }

                #(#fn_attrs)*
                #fn_vis fn #fn_name() -> #builder_ident {
                    #builder_ident::default()
                }

                fn #impl_ident(__tessera_props: &#props_ident) {
                    let __tessera_component_type_id: u64 = #component_type_id_tokens;
                    let __tessera_phase_guard = {
                        use #crate_path::__private::{RuntimePhase, push_phase};
                        push_phase(RuntimePhase::Build)
                    };
                    let __tessera_fn_name: &str = stringify!(#fn_name);
                    let __tessera_node_id = #register_tokens;

                    let _node_ctx_guard = {
                        use #crate_path::__private::push_current_node;
                        push_current_node(
                            __tessera_node_id,
                            __tessera_component_type_id,
                            __tessera_fn_name,
                        )
                    };

                    let __tessera_instance_key: u64 = #crate_path::__private::current_instance_key();
                    let __tessera_instance_logic_id: u64 =
                        #crate_path::__private::current_instance_logic_id();
                    let _instance_ctx_guard = {
                        use #crate_path::__private::push_current_component_instance_key;
                        push_current_component_instance_key(__tessera_instance_key)
                    };
                    let _component_scope_guard = {
                        struct ComponentScopeGuard;
                        impl Drop for ComponentScopeGuard {
                            fn drop(&mut self) {
                                #crate_path::__private::finish_component_node();
                            }
                        }
                        ComponentScopeGuard
                    };
                    #crate_path::__private::set_current_node_identity(
                        __tessera_instance_key,
                        __tessera_instance_logic_id,
                    );
                    #prop_assert_tokens
                    #crate_path::__private::record_current_context_snapshot_for(__tessera_instance_key);
                    #replay_tokens
                    if __tessera_component_reused {
                        return;
                    }
                    #(#field_resolutions)*
                    #fn_block
                }
            }
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
/// * Override: `#[shard(lifecycle = scope)]` to persist for the lifetime of the
///   current router controller hosted by `shard_home`
///
/// Route-scoped state is removed on route pop/clear. Scope-scoped state is
/// removed when the hosting `shard_home` is dropped.
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
///     fn destination_id() -> &'static str { "<module>::profile_page" }
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
/// * Routing helper: `tessera_shard::shard_home`
/// * Router controller internals: `tessera_shard::RouterController`
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

    let ui_crate_path: syn::Path = shard_args
        .crate_path
        .unwrap_or_else(|| syn::parse_quote!(::tessera_ui));
    let shard_crate_path: syn::Path = shard_args
        .shard_crate_path
        .unwrap_or_else(|| syn::parse_quote!(::tessera_shard));

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
    let router_params: Vec<_> = shard_params
        .iter()
        .filter(|param| param.is_router)
        .collect();
    if router_params.len() > 1 {
        return syn::Error::new_spanned(
            &func.sig.inputs,
            "#[shard] supports at most one `#[router]` parameter",
        )
        .to_compile_error()
        .into();
    }
    if let Some(router_param) = router_params.first()
        && !is_state_router_controller_type(&router_param.ty)
    {
        return syn::Error::new_spanned(
            &router_param.ty,
            "`#[router]` parameters must have type `State<RouterController>`",
        )
        .to_compile_error()
        .into();
    }

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
                quote! { #shard_crate_path::router::ShardStateLifeCycle::Scope }
            } else if lifecycle_name == "shard" {
                quote! { #shard_crate_path::router::ShardStateLifeCycle::Shard }
            } else {
                return syn::Error::new_spanned(
                    lifecycle,
                    "unsupported `lifecycle` in #[shard(...)]: expected `scope` or `shard`",
                )
                .to_compile_error()
                .into();
            }
        }
        None => quote! { #shard_crate_path::router::ShardStateLifeCycle::Shard },
    };

    let func_body = func.block;
    let func_name_str = func.sig.ident.to_string();

    let func_attrs = &func.attrs;
    let func_vis = &func.vis;
    let router_binding =
        router_params
            .first()
            .map_or_else(proc_macro2::TokenStream::new, |param| {
                let router_ident = &param.ident;
                quote! {
                    let #router_ident = #router_ident
                        .expect("`#[router]` injection is only available inside shard_home");
                }
            });
    let mut func_sig_modified = func.sig.clone();
    for input in &mut func_sig_modified.inputs {
        let FnArg::Typed(pat_type) = input else {
            continue;
        };
        let is_router = pat_type
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("router"));
        if !is_router {
            continue;
        }
        let original_ty = (*pat_type.ty).clone();
        *pat_type.ty = parse_quote!(::core::option::Option<#original_ty>);
        pat_type
            .attrs
            .retain(|attr| !attr.path().is_ident("router"));
        pat_type.attrs.push(syn::parse_quote!(#[prop(skip_setter)]));
        pat_type.attrs.push(
            syn::parse_quote!(#[default(Some(#shard_crate_path::__private::current_router_controller()))]),
        );
    }

    // Generate struct name for the new RouterDestination
    let func_name = func.sig.ident.clone();
    let struct_name = syn::Ident::new(
        &format!("{}Destination", func_name_str.to_upper_camel_case()),
        func_name.span(),
    );
    // Generate fields for the new struct that will implement `RouterDestination`
    let dest_fields = shard_params
        .iter()
        .filter(|param| !param.is_router)
        .map(|param| {
            let ident = &param.ident;
            let ty = &param.ty;
            quote! { pub #ident: #ty }
        });

    // Keep all explicit function parameters as destination props.
    let expanded = {
        let destination_builder_setters: Vec<_> = shard_params
            .iter()
            .filter(|param| !param.is_router)
            .map(|param| {
                let ident = &param.ident;
                if option_inner_type(&param.ty).is_some() {
                    quote! {
                        if let Some(value) = self.#ident.clone() {
                            __tessera_builder = __tessera_builder.#ident(value);
                        }
                    }
                } else {
                    quote! {
                        __tessera_builder = __tessera_builder.#ident(self.#ident.clone());
                    }
                }
            })
            .collect();

        if let Some(state_type) = state_type {
            quote! {
                #func_vis struct #struct_name {
                    #(#dest_fields),*
                }

                impl #shard_crate_path::router::RouterDestination for #struct_name {
                    fn exec_component(&self) {
                        let mut __tessera_builder = #func_name();
                        #(#destination_builder_setters)*
                        drop(__tessera_builder);
                    }

                    fn destination_id() -> &'static str {
                        concat!(module_path!(), "::", #func_name_str)
                    }
                }

                #(#func_attrs)*
                #[#ui_crate_path::tessera(#ui_crate_path)]
                #func_vis #func_sig_modified {
                    const SHARD_ID: &str = concat!(module_path!(), "::", #func_name_str);
                    #router_binding
                    #shard_crate_path::__private::with_current_router_shard_state::<#state_type, _, _>(
                        SHARD_ID,
                        #state_lifecycle_tokens,
                        |state| {
                            #func_body
                        },
                    )
                }
            }
        } else {
            quote! {
                #func_vis struct #struct_name {
                    #(#dest_fields),*
                }

                impl #shard_crate_path::router::RouterDestination for #struct_name {
                    fn exec_component(&self) {
                        let mut __tessera_builder = #func_name();
                        #(#destination_builder_setters)*
                        drop(__tessera_builder);
                    }

                    fn destination_id() -> &'static str {
                        concat!(module_path!(), "::", #func_name_str)
                    }
                }

                #(#func_attrs)*
                #[#ui_crate_path::tessera(#ui_crate_path)]
                #func_vis #func_sig_modified {
                    #router_binding
                    #func_body
                }
            }
        }
    };

    TokenStream::from(expanded)
}
