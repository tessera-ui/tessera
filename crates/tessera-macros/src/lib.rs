//! # Tessera Macros
//!
//! This crate provides procedural macros for the Tessera UI framework.
//! The main export is the `#[tessera]` attribute macro, which transforms
//! regular Rust functions into Tessera UI components.
#![deny(
    missing_docs,
    clippy::unwrap_used,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::invalid_html_tags
)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Block, Expr, FnArg, GenericArgument, GenericParam, Generics, Ident, ItemFn, Pat, Path,
    PathArguments, Type, parse_macro_input, parse_quote, visit_mut::VisitMut,
};

fn parse_crate_path(attr: proc_macro::TokenStream) -> syn::Result<syn::Path> {
    if attr.is_empty() {
        Ok(syn::parse_quote!(::tessera_ui))
    } else {
        let tokens: proc_macro2::TokenStream = attr.clone().into();
        syn::parse(attr).map_err(|_| {
            syn::Error::new_spanned(tokens, "expected a crate path like `crate` or `tessera_ui`")
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PropAttrConfig {
    into: bool,
    skip_setter: bool,
    skip_eq: bool,
    borrow: bool,
}
/// The builder-helper shape a component parameter gets from its declared type.
///
/// The parameter type is the single source of truth: functor parameters carry
/// their parsed functor signature so the generated setters never have to
/// re-inspect the type.
#[derive(Clone, Debug)]
enum PropHelperKind {
    Callback,
    CallbackWith { arg: Type, ret: Type },
    RenderSlot,
    RenderSlotWith { arg: Type },
}

fn parse_prop_attr(attrs: &[syn::Attribute]) -> syn::Result<PropAttrConfig> {
    let mut config = PropAttrConfig::default();
    for attr in attrs {
        if !attr.path().is_ident("prop") {
            continue;
        }

        match &attr.meta {
            syn::Meta::Path(_) => {
                return Err(syn::Error::new_spanned(
                    attr,
                    "expected `#[prop(...)]`; supported options are `into`, `skip_setter`, and `skip_eq`",
                ));
            }
            syn::Meta::List(_) => {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("into") {
                        if config.into {
                            return Err(meta.error("duplicate `into` in #[prop(...)]"));
                        }
                        config.into = true;
                        return Ok(());
                    }
                    if meta.path.is_ident("skip_setter") {
                        if config.skip_setter {
                            return Err(meta.error("duplicate `skip_setter` in #[prop(...)]"));
                        }
                        config.skip_setter = true;
                        return Ok(());
                    }
                    if meta.path.is_ident("skip_eq") {
                        if config.skip_eq {
                            return Err(meta.error("duplicate `skip_eq` in #[prop(...)]"));
                        }
                        config.skip_eq = true;
                        return Ok(());
                    }
                    // Framework layout primitives compare a borrowed candidate
                    // with stored node state; components receive owned props.
                    if meta.path.is_ident("borrow")
                        && std::env::var("CARGO_CRATE_NAME").as_deref() == Ok("tessera_ui")
                    {
                        if config.borrow {
                            return Err(meta.error("duplicate `borrow` in #[prop(...)]"));
                        }
                        config.borrow = true;
                        return Ok(());
                    }
                    if meta.path.is_ident("crate_path") {
                        return Err(meta.error(
                            "`crate_path` is only valid on the `#[tessera]` attribute, not on component parameters",
                        ));
                    }

                    Err(meta.error(
                        "unsupported #[prop(...)] option; supported options are `into`, `skip_setter`, and `skip_eq`",
                    ))
                })?;
            }
            syn::Meta::NameValue(_) => {
                return Err(syn::Error::new_spanned(
                    attr,
                    "unsupported `#[prop = ...]` form; use `#[prop(...)]`",
                ));
            }
        }
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
        let (arg, ret) = parse_functor_signature(&value_ty, "CallbackWith")?;
        return Some(PropHelperKind::CallbackWith { arg, ret });
    }
    if ident == "RenderSlot" {
        return Some(PropHelperKind::RenderSlot);
    }
    if ident == "RenderSlotWith" {
        let (arg, _ret) = parse_functor_signature(&value_ty, "RenderSlotWith")?;
        return Some(PropHelperKind::RenderSlotWith { arg });
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
struct ShardParam {
    ident: Ident,
    ty: Type,
    is_router: bool,
    is_model: bool,
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
                let is_router = is_state_router_controller_type(&pat_type.ty);
                let is_model = model_value_type(&pat_type.ty).is_some();
                params.push(ShardParam {
                    ident: pat_ident.ident.clone(),
                    ty: (*pat_type.ty).clone(),
                    is_router,
                    is_model,
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

        fields.push(PropFieldSpec {
            ident: pat_ident.ident.clone(),
            ty: (*arg.ty).clone(),
            attrs: parse_prop_attr(&arg.attrs)?,
            helper: infer_prop_helper_kind(arg.ty.as_ref()),
        });
    }

    Ok(ComponentPropSignature::Params(fields))
}

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
    attrs: PropAttrConfig,
    helper: Option<PropHelperKind>,
}

fn model_value_type(ty: &Type) -> Option<Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Model" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    let mut type_args = arguments.args.iter().filter_map(|arg| match arg {
        GenericArgument::Type(ty) => Some(ty.clone()),
        _ => None,
    });
    let value = type_args.next()?;
    type_args.next().is_none().then_some(value)
}

impl PropFieldSpec {
    fn is_model(&self) -> bool {
        model_value_type(&self.ty).is_some()
    }
}

fn is_required_component_field(field: &PropFieldSpec) -> bool {
    option_inner_type(&field.ty).is_none() && !field.is_model()
}

fn stored_value_ty(field_ty: &Type) -> Type {
    option_inner_type(field_ty).unwrap_or_else(|| field_ty.clone())
}

fn stored_field_ty(field_ty: &Type) -> Type {
    let value_ty = stored_value_ty(field_ty);
    parse_quote!(::core::option::Option<#value_ty>)
}

fn generate_constructor_param_and_assignment(
    field: &PropFieldSpec,
    crate_path: &Path,
) -> syn::Result<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    let ident = &field.ident;
    let field_ty = &field.ty;
    let helper_uses_into = field.helper.is_some();

    if let Some(helper) = &field.helper {
        match helper {
            PropHelperKind::CallbackWith { arg, ret } if is_unit_type(arg) => {
                let param = quote! {
                    #ident: impl Fn() -> #ret + 'static
                };
                let assignment = quote! {
                    __tessera_call.configure_deferred(stringify!(#ident), move |__tessera_pending_props| {
                        __tessera_pending_props.#ident = Some(#crate_path::CallbackWith::new(move |()| #ident()));
                    });
                };
                return Ok((param, assignment));
            }
            PropHelperKind::RenderSlotWith { arg } if is_unit_type(arg) => {
                let output = format_ident!("__Tessera{}Output", pascal_case_ident(ident, ""));
                let param = quote! {
                    #ident: impl Fn() -> #output + 'static
                };
                let assignment = quote! {
                    __tessera_call.configure_deferred(stringify!(#ident), move |__tessera_pending_props| {
                        __tessera_pending_props.#ident = Some(#crate_path::RenderSlotWith::new(move |()| #ident()));
                    });
                };
                return Ok((param, assignment));
            }
            PropHelperKind::Callback
            | PropHelperKind::CallbackWith { .. }
            | PropHelperKind::RenderSlot
            | PropHelperKind::RenderSlotWith { .. } => {}
        }
    }

    let constructor_param = if helper_uses_into {
        quote!(#ident: impl Into<#field_ty> + 'static)
    } else if field.attrs.into {
        quote!(#ident: impl Into<#field_ty>)
    } else {
        quote!(#ident: #field_ty)
    };
    let constructor_assignment = if helper_uses_into {
        quote! {
            __tessera_call.configure_deferred(stringify!(#ident), move |__tessera_pending_props| {
                __tessera_pending_props.#ident = Some(#ident.into());
            });
        }
    } else {
        let value = if field.attrs.into {
            quote!(#ident.into())
        } else {
            quote!(#ident)
        };
        quote! {
            let #ident = #value;
            __tessera_call.configure_value(stringify!(#ident), move |__tessera_pending_props| {
                ::core::mem::replace(&mut __tessera_pending_props.#ident, Some(#ident))
            });
        }
    };

    Ok((constructor_param, constructor_assignment))
}

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

fn hidden_const_marker_ident(fn_name: &Ident, const_ident: &Ident) -> Ident {
    format_ident!(
        "__Tessera{}ConstMarker{}",
        pascal_case_ident(fn_name, ""),
        pascal_case_ident(const_ident, "")
    )
}

fn validate_component_generics(generics: &Generics) -> syn::Result<()> {
    for param in &generics.params {
        if let GenericParam::Lifetime(param) = param {
            return Err(syn::Error::new_spanned(
                param,
                "#[tessera] components do not support lifetime generics; use owned props instead",
            ));
        }
    }
    Ok(())
}

fn generic_marker_tokens(
    fn_name: &Ident,
    generics: &Generics,
) -> (
    Vec<proc_macro2::TokenStream>,
    Option<proc_macro2::TokenStream>,
) {
    let mut const_marker_defs = Vec::new();
    let mut marker_parts = Vec::new();

    for param in &generics.params {
        match param {
            GenericParam::Type(param) => {
                let ident = &param.ident;
                marker_parts.push(quote!(#ident));
            }
            GenericParam::Const(param) => {
                let ident = &param.ident;
                let ty = &param.ty;
                let marker_ident = hidden_const_marker_ident(fn_name, ident);
                const_marker_defs.push(quote! {
                    struct #marker_ident<const #ident: #ty>;
                });
                marker_parts.push(quote!(#marker_ident<#ident>));
            }
            GenericParam::Lifetime(_) => unreachable!("lifetime generics are rejected earlier"),
        }
    }

    if marker_parts.is_empty() {
        return (const_marker_defs, None);
    }

    let marker_ty = if marker_parts.len() == 1 {
        let marker_part = &marker_parts[0];
        quote!(#marker_part)
    } else {
        quote!((#(#marker_parts,)*))
    };

    (
        const_marker_defs,
        Some(quote! {
            __tessera_generic_marker: ::core::marker::PhantomData<#marker_ty>
        }),
    )
}

fn generate_props_clone_default_impls(
    type_name: &Ident,
    generics: &Generics,
    field_idents: &[Ident],
    has_generic_marker: bool,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let clone_fields: Vec<_> = field_idents
        .iter()
        .map(|ident| quote!(#ident: self.#ident.clone()))
        .collect();
    let default_fields: Vec<_> = field_idents
        .iter()
        .map(|ident| quote!(#ident: ::core::default::Default::default()))
        .collect();
    let clone_marker = has_generic_marker.then(|| {
        quote! {
            __tessera_generic_marker: ::core::marker::PhantomData,
        }
    });
    let default_marker = has_generic_marker.then(|| {
        quote! {
            __tessera_generic_marker: ::core::marker::PhantomData,
        }
    });

    quote! {
        impl #impl_generics ::core::clone::Clone for #type_name #ty_generics #where_clause {
            fn clone(&self) -> Self {
                Self {
                    #(#clone_fields,)*
                    #clone_marker
                }
            }
        }

        impl #impl_generics ::core::default::Default for #type_name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(#default_fields,)*
                    #default_marker
                }
            }
        }
    }
}

fn generate_prop_like_impls(
    type_name: &Ident,
    component_impl: &Ident,
    generics: &Generics,
    fields: &[PropFieldSpec],
    crate_path: &Path,
) -> proc_macro2::TokenStream {
    let compare_fields: Vec<_> = fields
        .iter()
        .filter(|field| !field.attrs.skip_eq)
        .map(|field| field_compare_expr(&field.ident, &stored_field_ty(&field.ty)))
        .collect();
    let prop_eq_expr = if compare_fields.is_empty() {
        quote! { true }
    } else {
        quote! { true #(&& #compare_fields)* }
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::core::cmp::PartialEq for #type_name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                #prop_eq_expr
            }
        }

        impl #impl_generics #crate_path::__private::Prop for #type_name #ty_generics #where_clause {
            fn prop_eq(&self, other: &Self) -> bool {
                <Self as ::core::cmp::PartialEq>::eq(self, other)
            }

            fn run_component(self: ::std::rc::Rc<Self>) {
                #component_impl(&self);
            }
        }
    }
}

fn generate_builder_methods(
    fields: &[PropFieldSpec],
    crate_path: &syn::Path,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut methods = Vec::new();
    for field in fields {
        if field.attrs.skip_setter || is_required_component_field(field) {
            continue;
        }
        let ident = &field.ident;
        let ty = stored_value_ty(&field.ty);
        let method_doc = format!("Configure `{ident}` before this component executes.");
        if let Some(helper) = &field.helper {
            let shared = format_ident!("{}_shared", ident);
            let (generics, bound, value) = match helper {
                PropHelperKind::Callback => (
                    quote!(<F>),
                    quote!(F: Fn() + 'static),
                    quote!(#crate_path::Callback::new(value)),
                ),
                PropHelperKind::CallbackWith { arg, ret } if is_unit_type(arg) => (
                    quote!(<F>),
                    quote!(F: Fn() -> #ret + 'static),
                    quote!(#crate_path::CallbackWith::new(move |()| value())),
                ),
                PropHelperKind::CallbackWith { arg, ret } => (
                    quote!(<F>),
                    quote!(F: Fn(#arg) -> #ret + 'static),
                    quote!(#crate_path::CallbackWith::new(value)),
                ),
                PropHelperKind::RenderSlot => (
                    quote!(<F, Output>),
                    quote!(F: Fn() -> Output + 'static),
                    quote!(#crate_path::RenderSlot::new(value)),
                ),
                PropHelperKind::RenderSlotWith { arg } if is_unit_type(arg) => (
                    quote!(<F, Output>),
                    quote!(F: Fn() -> Output + 'static),
                    quote!(#crate_path::RenderSlotWith::new(move |()| value())),
                ),
                PropHelperKind::RenderSlotWith { arg } => (
                    quote!(<F, Output>),
                    quote!(F: Fn(#arg) -> Output + 'static),
                    quote!(#crate_path::RenderSlotWith::new(value)),
                ),
            };
            methods.push(quote! {
                #[doc = #method_doc]
                #[track_caller]
                pub fn #ident #generics (&self, value: F) -> Self where #bound {
                    self.props.configure_deferred(stringify!(#ident), move |props| {
                        props.#ident = Some(#value);
                    });
                    Self { props: self.props }
                }
                #[doc = #method_doc]
                #[track_caller]
                pub fn #shared(&self, value: impl Into<#ty> + 'static) -> Self {
                    self.props.configure_deferred(stringify!(#ident), move |props| props.#ident = Some(value.into()));
                    Self { props: self.props }
                }
            });
        } else {
            let parameter = if field.attrs.into {
                quote!(impl Into<#ty>)
            } else {
                quote!(#ty)
            };
            let value = if field.attrs.into {
                quote!(value.into())
            } else {
                quote!(value)
            };
            methods.push(quote! {
                #[doc = #method_doc]
                #[track_caller]
                pub fn #ident(&self, value: #parameter) -> Self {
                    let value = #value;
                    self.props.configure_value(stringify!(#ident), move |props| {
                        ::core::mem::replace(&mut props.#ident, Some(value))
                    });
                    Self { props: self.props }
                }
            });
        }
        let optional = format_ident!("{}_optional", ident);
        methods.push(quote! {
            #[doc = #method_doc]
            #[track_caller]
            pub fn #optional(&self, value: Option<#ty>) -> Self {
                self.props.configure_value(stringify!(#ident), move |props| {
                    ::core::mem::replace(&mut props.#ident, value)
                });
                Self { props: self.props }
            }
        });
    }
    Ok(methods)
}

fn component_type_id_tokens(
    fn_name: &syn::Ident,
    prop_type: &syn::Type,
) -> proc_macro2::TokenStream {
    quote! {
        {
            use std::{any::TypeId, hash::{Hash, Hasher}};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            module_path!().hash(&mut hasher);
            stringify!(#fn_name).hash(&mut hasher);
            TypeId::of::<#prop_type>().hash(&mut hasher);
            hasher.finish()
        }
    }
}

struct ControlFlowInstrumenter {
    counter: u64,
}

impl ControlFlowInstrumenter {
    fn new() -> Self {
        Self { counter: 1 }
    }

    fn next_group_id(&mut self) -> u64 {
        let id = self.counter;
        self.counter += 1;
        id
    }

    fn wrap_expr_in_group(&mut self, expr: &mut Expr) {
        self.visit_expr_mut(expr);
        let group_id = self.next_group_id();
        // Use fully-qualified path ::tessera_ui to avoid relying on a crate
        // alias
        let original_expr = &expr;
        let new_expr: Expr = parse_quote! {
            {
                let _group_guard = ::tessera_ui::__private::GroupGuard::new(#group_id);
                #original_expr
            }
        };
        *expr = new_expr;
    }

    fn wrap_block_in_group(&mut self, block: &mut Block) {
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
/// # See Also
///
/// * [`#[shard]`](crate::shard) for components that are navigation destinations.
///
/// A `Model<T>` parameter is injected: callers may pass one, otherwise the
/// nearest `ModelStore` (or the component itself) owns a model created by
/// `ModelInit::init`.
#[proc_macro_attribute]
pub fn tessera(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        let attr = proc_macro2::TokenStream::from(attr);
        return syn::Error::new_spanned(attr, "#[tessera] takes no arguments")
            .to_compile_error()
            .into();
    }
    let crate_path: Path = parse_quote!(::tessera_ui);
    let begin_scope = quote!(#crate_path::__private::begin_component_node);

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
        || !matches!(input_fn.sig.safety, syn::Safety::Default)
        || input_fn.sig.abi.is_some()
        || input_fn.sig.variadic.is_some()
    {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "#[tessera] components must be plain free functions",
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
    if let Err(err) = validate_component_generics(&input_fn.sig.generics) {
        return err.to_compile_error().into();
    }
    let prop_signature = match strict_prop_signature(&input_fn.sig) {
        Ok(v) => v,
        Err(err) => return err.to_compile_error().into(),
    };

    let mut instrumenter = ControlFlowInstrumenter::new();
    instrumenter.visit_block_mut(&mut input_fn.block);
    let fn_block = &input_fn.block;
    let generics = &input_fn.sig.generics;
    let mut item_generics = generics.clone();
    item_generics.where_clause = None;
    let where_clause = &generics.where_clause;
    let (impl_generics, ty_generics, _) = generics.split_for_impl();

    let fields = match &prop_signature {
        ComponentPropSignature::Unit => &[][..],
        ComponentPropSignature::Params(fields) => fields.as_slice(),
    };
    let expanded = {
        let props_ident = hidden_props_ident(fn_name);
        let builder_ident = builder_ident(fn_name);
        let impl_ident = hidden_component_impl_ident(fn_name);
        let (const_marker_defs, marker_field_def) = generic_marker_tokens(fn_name, generics);
        let has_generic_marker = marker_field_def.is_some();
        let prop_type: syn::Type = syn::parse_quote!(#props_ident #ty_generics);
        let uncached_type_id = component_type_id_tokens(fn_name, &prop_type);
        let (metadata, component_type_id_tokens) = if generics.params.is_empty() {
            (
                quote! {
                    impl #props_ident {
                        fn __tessera_type_id() -> u64 {
                            static TYPE_ID: ::std::sync::OnceLock<u64> = ::std::sync::OnceLock::new();
                            *TYPE_ID.get_or_init(|| #uncached_type_id)
                        }
                    }
                },
                quote! { #props_ident::__tessera_type_id() },
            )
        } else {
            (quote! {}, uncached_type_id)
        };
        let field_idents: Vec<_> = fields.iter().map(|field| field.ident.clone()).collect();
        let required_fields: Vec<_> = fields
            .iter()
            .filter(|field| is_required_component_field(field))
            .collect();
        let mut entry_generics = item_generics.clone();
        for field in &required_fields {
            if matches!(&field.helper, Some(PropHelperKind::RenderSlotWith { arg }) if is_unit_type(arg))
            {
                let output =
                    format_ident!("__Tessera{}Output", pascal_case_ident(&field.ident, ""));
                entry_generics.params.push(syn::parse_quote!(#output));
            }
        }
        // Conversions can run user code after registration. Preserve their
        // order relative to later property assignments and their destructors.
        let initial_count = required_fields
            .iter()
            .take_while(|field| field.helper.is_none() && !field.attrs.into)
            .count();
        let (initial_fields, pending_fields) = required_fields.split_at(initial_count);
        let constructor_params_and_assignments: Vec<_> = match pending_fields
            .iter()
            .map(|field| generate_constructor_param_and_assignment(field, &crate_path))
            .collect::<syn::Result<Vec<_>>>()
        {
            Ok(entries) => entries,
            Err(err) => return err.to_compile_error().into(),
        };
        let constructor_params: Vec<_> = initial_fields
            .iter()
            .map(|field| {
                let ident = &field.ident;
                let ty = &field.ty;
                quote!(#ident: #ty)
            })
            .chain(
                constructor_params_and_assignments
                    .iter()
                    .map(|(param, _)| param.clone()),
            )
            .collect();
        let constructor_assignments: Vec<_> = constructor_params_and_assignments
            .iter()
            .map(|(_, assignment)| assignment.clone())
            .collect();
        let initial_props = if initial_fields.is_empty() {
            quote!(<#props_ident #ty_generics>::default())
        } else {
            let initializers = initial_fields.iter().map(|field| {
                let ident = &field.ident;
                quote!(#ident: Some(#ident),)
            });
            quote!(#props_ident {
                #(#initializers)*
                ..<#props_ident #ty_generics>::default()
            })
        };
        let field_defs: Vec<_> = fields
            .iter()
            .map(|field| {
                let ident = &field.ident;
                let ty = stored_field_ty(&field.ty);
                quote!(#ident: #ty)
            })
            .collect();
        let field_resolutions: Vec<_> = fields
            .iter()
            .map(|field| {
                let ident = &field.ident;
                let ty = &field.ty;
                if field.is_model() {
                    let Some(model_ty) = model_value_type(ty) else {
                        unreachable!("model fields have a `Model<T>` type");
                    };
                    let model_index = fields
                        .iter()
                        .position(|candidate| candidate.ident == field.ident)
                        .unwrap_or_default() as u64;
                    return quote! {
                        let #ident: #ty = __tessera_props.#ident.clone().unwrap_or_else(|| {
                            #crate_path::__private::inject_model::<#model_ty>(#model_index)
                        });
                    };
                }
                if field.attrs.borrow {
                    let missing_prop_message =
                        format!("missing prop `{}` for component `{}`", ident, fn_name);
                    return quote! {
                        let #ident = __tessera_props.#ident.as_ref().unwrap_or_else(|| {
                            panic!(#missing_prop_message)
                        });
                    };
                }
                if option_inner_type(ty).is_some() {
                    return quote! {
                        let #ident: #ty = __tessera_props.#ident.clone();
                    };
                }

                let missing_prop_message = format!(
                    "missing required prop `{}` for component `{}`",
                    ident, fn_name
                );
                quote! {
                    let #ident: #ty = __tessera_props.#ident.clone().unwrap_or_else(|| {
                        panic!(#missing_prop_message)
                    });
                }
            })
            .collect();
        let prop_impl_tokens =
            generate_prop_like_impls(&props_ident, &impl_ident, generics, fields, &crate_path);
        let builder_methods = match generate_builder_methods(fields, &crate_path) {
            Ok(methods) => methods,
            Err(err) => return err.to_compile_error().into(),
        };
        let prop_assert_tokens = prop_assert_tokens(&crate_path, &prop_type);
        let marker_field_def = marker_field_def
            .map(|field| quote!(#field,))
            .unwrap_or_default();
        let props_clone_default_impls = generate_props_clone_default_impls(
            &props_ident,
            generics,
            &field_idents,
            has_generic_marker,
        );

        quote! {
            #(#const_marker_defs)*

            struct #props_ident #item_generics #where_clause {
                #(#field_defs,)*
                #marker_field_def
            }

            #props_clone_default_impls
            #prop_impl_tokens
            #metadata

            #[doc = concat!("Configuration handle returned by [`", stringify!(#fn_name), "`].")]
            #fn_vis struct #builder_ident #item_generics #where_clause {
                props: #crate_path::__private::ComponentCall<#props_ident #ty_generics>,
            }

            impl #impl_generics #builder_ident #ty_generics #where_clause {
                #(#builder_methods)*
            }

            #(#fn_attrs)*
            #fn_vis fn #fn_name #entry_generics(
                #(#constructor_params),*
            ) -> #builder_ident #ty_generics #where_clause {
                let __tessera_call = #crate_path::__private::register_component_call(
                        stringify!(#fn_name), #component_type_id_tokens,
                        #initial_props,
                    );
                #(#constructor_assignments)*
                #builder_ident { props: __tessera_call }
            }

            fn #impl_ident #item_generics(__tessera_props: &::std::rc::Rc<#props_ident #ty_generics>) #where_clause {
                #prop_assert_tokens
                let __tessera_scope = #begin_scope(
                    stringify!(#fn_name),
                    #component_type_id_tokens,
                    __tessera_props,
                );
                if __tessera_scope.is_retained() {
                    return;
                }
                #crate_path::__private::with_pending_scope(|| {
                    #crate_path::__private::record_component_execution(
                        stringify!(#fn_name),
                        __tessera_scope.instance_logic_id(),
                    );
                    #(#field_resolutions)*
                    #fn_block;
                });
            }
        }
    };

    TokenStream::from(expanded)
}

/// Sets Tessera lint levels for an item or inline module tree.
///
/// Use `#[tessera_ui::lint(allow(tessera::redundant_render_slot))]` with
/// `allow`, `warn`, `deny`, or `forbid`. Lints begin at their default levels;
/// nested item settings override inherited levels unless forbidden.
/// `cargo tessera check` applies these settings with rustc lint-level precedence,
/// including command-line `--force-warn` and `--cap-lints` controls.
#[proc_macro_attribute]
pub fn lint(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Generates platform-specific entry points from a shared `run` function.
///
/// # Usage
///
/// Annotate a public zero-argument function that returns
/// `tessera_ui::EntryPoint`.
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
/// the routing system.
///
/// # Features
///
/// * Generates a `StructNameDestination` (UpperCamelCase + `Destination`)
///   implementing `tessera_shard::router::RouterDestination`
/// * Other parameters become destination fields.
/// * A `State<RouterController>` parameter receives the router rendering the
///   page and is not a destination field.
/// * `Model<T>` parameters are injected like in any component and are not
///   destination fields; the route entry provides the model store, so a model
///   lives until its route is popped.
#[cfg(feature = "shard")]
#[proc_macro_attribute]
pub fn shard(attr: TokenStream, input: TokenStream) -> TokenStream {
    use heck::ToUpperCamelCase;
    if !attr.is_empty() {
        return syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "#[shard] takes no arguments; inject a typed `Model<T>` parameter instead",
        )
        .to_compile_error()
        .into();
    }

    let ui_crate_path: syn::Path = syn::parse_quote!(::tessera_ui);
    let shard_crate_path: syn::Path = syn::parse_quote!(::tessera_shard);

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
            "#[shard] supports at most one `State<RouterController>` parameter",
        )
        .to_compile_error()
        .into();
    }

    let func_body = func.block;
    let func_name_str = func.sig.ident.to_string();

    let func_attrs = &func.attrs;
    let func_vis = &func.vis;
    let router_binding = router_params.first().map(|param| {
        let ident = &param.ident;
        let ty = &param.ty;
        quote! {
            let #ident: #ty = #ui_crate_path::use_context::<#ty>()
                .expect("shard pages render inside `shard_home`")
                .get();
        }
    });
    let mut func_sig_modified = func.sig.clone();
    func_sig_modified.inputs = func_sig_modified
        .inputs
        .into_iter()
        .filter(|input| {
            !matches!(input, FnArg::Typed(pat_type) if is_state_router_controller_type(&pat_type.ty))
        })
        .collect();

    // Generate struct name for the new RouterDestination
    let func_name = func.sig.ident.clone();
    let struct_name = syn::Ident::new(
        &format!("{}Destination", func_name_str.to_upper_camel_case()),
        func_name.span(),
    );
    // Generate fields for the new struct that will implement
    // `RouterDestination`
    let dest_fields = shard_params
        .iter()
        .filter(|param| !param.is_router && !param.is_model)
        .map(|param| {
            let ident = &param.ident;
            let ty = &param.ty;
            quote! { pub #ident: #ty }
        });

    let expanded = {
        let required_args = shard_params
            .iter()
            .filter(|param| {
                !param.is_router && !param.is_model && option_inner_type(&param.ty).is_none()
            })
            .map(|param| {
                let ident = &param.ident;
                quote! { self.#ident.clone() }
            });
        let destination_builder_setters = shard_params
            .iter()
            .filter(|param| {
                !param.is_router && !param.is_model && option_inner_type(&param.ty).is_some()
            })
            .map(|param| {
                let ident = &param.ident;
                quote! {
                    if let Some(value) = self.#ident.clone() {
                        __tessera_builder.#ident(value);
                    }
                }
            });

        {
            quote! {
                #[derive(Clone, PartialEq)]
                #func_vis struct #struct_name {
                    #(#dest_fields),*
                }

                impl #shard_crate_path::router::RouterDestination for #struct_name {
                    fn render(&self) {
                        let __tessera_builder = #func_name(#(#required_args),*);
                        #(#destination_builder_setters)*
                    }
                }

                #(#func_attrs)*
                #[#ui_crate_path::tessera]
                #func_vis #func_sig_modified {
                    #router_binding
                    #func_body
                }
            }
        }
    };

    TokenStream::from(expanded)
}
