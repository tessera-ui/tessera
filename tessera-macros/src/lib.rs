//! # Tessera Macros
//!
//! This crate provides procedural macros for the Tessera UI framework.
//! The main export is the `#[tessera]` attribute macro, which transforms
//! regular Rust functions into Tessera UI components.

use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Block, Expr, FnArg, ItemFn, Pat, Type, parse_macro_input, parse_quote, visit_mut::VisitMut,
};

/// Helper: parse crate path from attribute TokenStream
fn parse_crate_path(attr: proc_macro::TokenStream) -> syn::Path {
    if attr.is_empty() {
        // Default to `tessera_ui` if no path is provided
        syn::parse_quote!(::tessera_ui)
    } else {
        // Parse the provided path, e.g., `crate` or `tessera_ui`
        syn::parse(attr).expect("Expected a valid path like `crate` or `tessera_ui`")
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
                        input_handler_fn: None,
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

    let arg = sig.inputs.first().expect("single input already validated");
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

/// Helper: tokens to inject `input_handler`
fn input_handler_inject_tokens(crate_path: &syn::Path) -> proc_macro2::TokenStream {
    quote! {
        #[allow(clippy::needless_pass_by_value)]
        fn input_handler<F>(fun: F)
        where
            F: Fn(#crate_path::InputHandlerInput) + Send + Sync + 'static,
        {
            use #crate_path::InputHandlerFn;
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .current_node_mut()
                    .unwrap()
                    .input_handler_fn = Some(Box::new(fun) as Box<InputHandlerFn>)
            });
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
        self.wrap_block_in_group(&mut f.body);
    }

    fn visit_expr_while_mut(&mut self, w: &mut syn::ExprWhile) {
        self.visit_expr_mut(&mut w.cond);
        self.wrap_block_in_group(&mut w.body);
    }

    fn visit_expr_loop_mut(&mut self, l: &mut syn::ExprLoop) {
        self.wrap_block_in_group(&mut l.body);
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
    let crate_path: syn::Path = parse_crate_path(attr);

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
    let crate_path: syn::Path = parse_crate_path(attr);
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
/// * (Optional) Injects a single `#[state]` parameter whose type:
///   - Must implement `Default + Send + Sync + 'static`
///   - Is constructed (or reused) and passed to your function body
/// * Produces a stable shard ID: `module_path!()::function_name`
///
/// # Lifecycle
///
/// Controlled by the generated state injection (via `#[state(...)]`).
/// * Default: `Shard` – state is removed when the destination is `pop()`‑ed
/// * Override: `#[state(scope)]` – persist for the lifetime of current
///   `router_scope`
///
/// Route-scoped state is removed on route pop/clear. Scope-scoped state is
/// removed when the router scope is dropped.
///
/// # Parameter Transformation
///
/// * At most one parameter may be annotated with `#[state]`.
/// * That parameter is removed from the *generated* function signature and
///   supplied implicitly.
/// * All other parameters remain explicit and become public fields on the
///   generated `*Destination` struct.
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
/// * No support for multiple `#[state]` params (compile panic if violated)
/// * Do not manually implement `RouterDestination` for these pages; rely on
///   generation
///
/// # See Also
///
/// * Routing helpers: `tessera_ui::router::{Router, router_root}`
/// * Scoped router internals: `tessera_shard::router::Router`
///
/// # Errors / Panics
///
/// * Panics at compile time if multiple `#[state]` parameters are used or
///   unsupported pattern forms are encountered.
#[cfg(feature = "shard")]
#[proc_macro_attribute]
pub fn shard(attr: TokenStream, input: TokenStream) -> TokenStream {
    use heck::ToUpperCamelCase;
    use syn::Pat;

    let crate_path: syn::Path = if attr.is_empty() {
        syn::parse_quote!(::tessera_ui)
    } else {
        syn::parse(attr).expect("Expected a valid path like `crate` or `tessera_ui`")
    };

    let mut func = parse_macro_input!(input as ItemFn);
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

    // Handle #[state] parameters, ensuring it's unique and removing it from
    // the signature. Also parse optional lifecycle argument:
    // #[state(scope)] or #[state(shard)].
    let mut state_param = None;
    let mut state_lifecycle: Option<proc_macro2::TokenStream> = None;
    let mut new_inputs = syn::punctuated::Punctuated::new();
    for arg in func.sig.inputs.iter() {
        if let syn::FnArg::Typed(pat_type) = arg {
            // Detect #[state] and parse optional argument
            let mut is_state = false;
            let mut lifecycle_override: Option<proc_macro2::TokenStream> = None;
            for attr in &pat_type.attrs {
                if attr.path().is_ident("state") {
                    is_state = true;
                    // Try parse an optional argument: #[state(scope)] /
                    // #[state(shard)]
                    if let Ok(arg_ident) = attr.parse_args::<syn::Ident>() {
                        let s = arg_ident.to_string().to_lowercase();
                        if s == "scope" {
                            lifecycle_override = Some(
                                quote! { #crate_path::tessera_shard::ShardStateLifeCycle::Scope },
                            );
                        } else if s == "shard" {
                            lifecycle_override = Some(
                                quote! { #crate_path::tessera_shard::ShardStateLifeCycle::Shard },
                            );
                        } else {
                            panic!(
                                "Unsupported #[state(...)] argument in #[shard]: expected `scope` or `shard`"
                            );
                        }
                    }
                }
            }
            if is_state {
                if state_param.is_some() {
                    panic!(
                        "#[shard] function must have at most one parameter marked with #[state]."
                    );
                }
                state_param = Some(pat_type.clone());
                state_lifecycle = lifecycle_override;
                continue;
            }
        }
        new_inputs.push(arg.clone());
    }
    func.sig.inputs = new_inputs;

    let (state_name, state_type) = if let Some(state_param) = state_param {
        let name = match *state_param.pat {
            Pat::Ident(ref pat_ident) => pat_ident.ident.clone(),
            _ => panic!(
                "Unsupported parameter pattern in #[shard] function. Please use a simple identifier like `state`."
            ),
        };
        (Some(name), Some(state_param.ty))
    } else {
        (None, None)
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
    let dest_fields = func.sig.inputs.iter().map(|arg| match arg {
        syn::FnArg::Typed(pat_type) => {
            let ident = match *pat_type.pat {
                syn::Pat::Ident(ref pat_ident) => &pat_ident.ident,
                _ => panic!("Unsupported parameter pattern in #[shard] function."),
            };
            let ty = &pat_type.ty;
            quote! { pub #ident: #ty }
        }
        _ => panic!("Unsupported parameter type in #[shard] function."),
    });

    // Only keep the parameters that are not marked with #[state]
    let param_idents: Vec<_> = func
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => match *pat_type.pat {
                syn::Pat::Ident(ref pat_ident) => pat_ident.ident.clone(),
                _ => panic!("Unsupported parameter pattern in #[shard] function."),
            },
            _ => panic!("Unsupported parameter type in #[shard] function."),
        })
        .collect();

    let shard_prop_fields = func.sig.inputs.iter().map(|arg| match arg {
        syn::FnArg::Typed(pat_type) => {
            let ident = match *pat_type.pat {
                syn::Pat::Ident(ref pat_ident) => &pat_ident.ident,
                _ => panic!("Unsupported parameter pattern in #[shard] function."),
            };
            let ty = &pat_type.ty;
            quote! { #ident: #ty }
        }
        _ => panic!("Unsupported parameter type in #[shard] function."),
    });

    let state_lifecycle_tokens = state_lifecycle.clone().unwrap_or_else(|| {
        quote! { #crate_path::tessera_shard::ShardStateLifeCycle::Shard }
    });

    let expanded = {
        // `exec_component` only passes struct fields (unmarked parameters).
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
            let state_name = state_name.as_ref().unwrap();
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
                            |#state_name| {
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
