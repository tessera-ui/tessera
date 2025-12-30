//! # Tessera Macros
//!
//! This crate provides procedural macros for the Tessera UI framework.
//! The main export is the `#[tessera]` attribute macro, which transforms
//! regular Rust functions into Tessera UI components.

use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro::TokenStream;
use quote::quote;
use syn::{Block, Expr, ItemFn, parse_macro_input, parse_quote, visit_mut::VisitMut};

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
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime.component_tree.add_node(
                    ComponentNode {
                        fn_name: stringify!(#fn_name).to_string(),
                        logic_id: __tessera_logic_id,
                        measure_fn: None,
                        input_handler_fn: None,
                    }
                )
            })
        }
    }
}

/// Helper: tokens to inject `measure`
fn measure_inject_tokens(crate_path: &syn::Path) -> proc_macro2::TokenStream {
    quote! {
        #[allow(clippy::needless_pass_by_value)]
        fn measure<F>(fun: F)
        where
            F: Fn(&#crate_path::MeasureInput<'_>) -> Result<#crate_path::ComputedData, #crate_path::MeasurementError>
                + Send
                + Sync
                + 'static,
        {
            use #crate_path::MeasureFn;
            use #crate_path::runtime::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .current_node_mut()
                    .unwrap()
                    .measure_fn = Some(Box::new(fun) as Box<MeasureFn>)
            });
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

/// Helper: tokens to inject `on_minimize`
fn on_minimize_inject_tokens(crate_path: &syn::Path) -> proc_macro2::TokenStream {
    quote! {
        let on_minimize = {
            use #crate_path::runtime::TesseraRuntime;
            |fun: Box<dyn Fn(bool) + Send + Sync + 'static>| {
                TesseraRuntime::with_mut(|runtime| runtime.on_minimize(fun));
            }
        };
    }
}

/// Helper: tokens to inject `on_close`
fn on_close_inject_tokens(crate_path: &syn::Path) -> proc_macro2::TokenStream {
    quote! {
        let on_close = {
            use #crate_path::runtime::TesseraRuntime;
            |fun: Box<dyn Fn() + Send + Sync + 'static>| {
                TesseraRuntime::with_mut(|runtime| runtime.on_close(fun));
            }
        };
    }
}

/// Helper: tokens to compute a stable logic id based on module path + function
/// name.
fn logic_id_tokens(fn_name: &syn::Ident) -> proc_macro2::TokenStream {
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
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_sig = &input_fn.sig;

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
    let measure_tokens = measure_inject_tokens(&crate_path);
    let state_tokens = input_handler_inject_tokens(&crate_path);
    let on_minimize_tokens = on_minimize_inject_tokens(&crate_path);
    let on_close_tokens = on_close_inject_tokens(&crate_path);
    let logic_id_tokens = logic_id_tokens(fn_name);

    // Generate the transformed function with Tessera runtime integration
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            let __tessera_logic_id: u64 = #logic_id_tokens;
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
                        TesseraRuntime::with_mut(|runtime| runtime.component_tree.pop_node());
                    }
                }
                ComponentScopeGuard
            };

            // Track current node for control-flow instrumentation
            let _node_ctx_guard = {
                use #crate_path::runtime::push_current_node;
                push_current_node(__tessera_node_id, __tessera_logic_id, __tessera_fn_name)
            };

            // Inject helper tokens
            #measure_tokens
            #state_tokens
            #on_minimize_tokens
            #on_close_tokens

            // Execute user's function body
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// Transforms a function into a *shard component* that can be navigated to via
/// the routing system and (optionally) provided with a lazily‑initialized
/// per‑shard state.
///
/// # Features
///
/// * Generates a `StructNameDestination` (UpperCamelCase + `Destination`)
///   implementing `tessera_ui_shard::router::RouterDestination`
/// * (Optional) Injects a single `#[state]` parameter whose type:
///   - Must implement `Default + Send + Sync + 'static`
///   - Is constructed (or reused) and passed to your function body
/// * Produces a stable shard ID: `module_path!()::function_name`
///
/// # Lifecycle
///
/// Controlled by the generated destination (via `#[state(...)]`).
/// * Default: `Shard` – state is removed when the destination is `pop()`‑ed
/// * Override: `#[state(app)]` (or `#[state(application)]`) – persist for the
///   entire application
///
/// When `pop()` is called and the destination lifecycle is `Shard`, the
/// registry entry is removed, freeing the state.
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
/// * Routing helpers: `tessera_ui::router::{push, pop, router_root}`
/// * Shard state registry: `tessera_ui_shard::ShardRegistry`
///
/// # Safety
///
/// Internally uses an unsafe cast inside the registry to recover `Arc<T>` from
/// `Arc<dyn ShardState>`; this is encapsulated and not exposed.
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

    // Handle #[state] parameters, ensuring it's unique and removing it from the
    // signature Also parse optional lifecycle argument: #[state(app)] or
    // #[state(shard)]
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
                    // Try parse an optional argument: #[state(app)] / #[state(shard)]
                    if let Ok(arg_ident) = attr.parse_args::<syn::Ident>() {
                        let s = arg_ident.to_string().to_lowercase();
                        if s == "app" || s == "application" {
                            lifecycle_override = Some(
                                quote! { #crate_path::tessera_ui_shard::ShardStateLifeCycle::Application },
                            );
                        } else if s == "shard" {
                            lifecycle_override = Some(
                                quote! { #crate_path::tessera_ui_shard::ShardStateLifeCycle::Shard },
                            );
                        } else {
                            panic!(
                                "Unsupported #[state(...)] argument in #[shard]: expected `app` or `shard`"
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

    let lifecycle_method_tokens = if let Some(lc) = state_lifecycle.clone() {
        quote! {
            fn life_cycle(&self) -> #crate_path::tessera_ui_shard::ShardStateLifeCycle {
                #lc
            }
        }
    } else {
        // Default is `Shard` per RouterDestination trait; no override needed.
        quote! {}
    };

    let expanded = {
        // `exec_component` only passes struct fields (unmarked parameters).
        let exec_args = param_idents
            .iter()
            .map(|ident| quote! { self.#ident.clone() });

        if let Some(state_type) = state_type {
            let state_name = state_name.as_ref().unwrap();
            quote! {
                #func_vis struct #struct_name {
                    #(#dest_fields),*
                }

                impl #crate_path::tessera_ui_shard::router::RouterDestination for #struct_name {
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

                    #lifecycle_method_tokens
                }

                #(#func_attrs)*
                #func_vis #func_sig_modified {
                    // Generate a stable unique ID at the call site
                    const SHARD_ID: &str = concat!(module_path!(), "::", #func_name_str);

                    // Call the global registry and pass the original function body as a closure
                    unsafe {
                        #crate_path::tessera_ui_shard::ShardRegistry::get().init_or_get::<#state_type, _, _>(
                            SHARD_ID,
                            |#state_name| {
                                #func_body
                            },
                        )
                    }
                }
            }
        } else {
            quote! {
                #func_vis struct #struct_name {
                    #(#dest_fields),*
                }

                impl #crate_path::tessera_ui_shard::router::RouterDestination for #struct_name {
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

                    #lifecycle_method_tokens
                }

                #(#func_attrs)*
                #func_vis #func_sig_modified {
                    #func_body
                }
            }
        }
    };

    TokenStream::from(expanded)
}
