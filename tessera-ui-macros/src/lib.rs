//! # Tessera Macros
//!
//! This crate provides procedural macros for the Tessera UI framework.
//! The main export is the `#[tessera]` attribute macro, which transforms
//! regular Rust functions into Tessera UI components.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use tessera_ui::tessera;
//!
//! #[tessera]
//! fn my_component() {
//!     // Component logic here
//!     // The macro provides access to `measure`, `state_handler` and `on_minimize` functions
//! }
//! ```
//!
//! The `#[tessera]` macro automatically:
//!
//! - Registers the function as a component in the Tessera component tree
//! - Injects `measure`, `state_handler` and `on_minimize` functions into the component scope
//! - Handles component tree management (adding/removing nodes)
//! - Provides error safety by wrapping the function body

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

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
            use #crate_path::{TesseraRuntime, ComponentNode};

            TesseraRuntime::with_mut(|runtime| {
                runtime.component_tree.add_node(
                    ComponentNode {
                        fn_name: stringify!(#fn_name).to_string(),
                        measure_fn: None,
                        state_handler_fn: None,
                    }
                )
            });
        }
    }
}

/// Helper: tokens to inject `measure`
fn measure_inject_tokens(crate_path: &syn::Path) -> proc_macro2::TokenStream {
    quote! {
        let measure = {
            use #crate_path::{MeasureFn, TesseraRuntime};
            |fun: Box<MeasureFn>| {
                TesseraRuntime::with_mut(|runtime| {
                    runtime
                        .component_tree
                        .current_node_mut()
                        .unwrap()
                        .measure_fn = Some(fun)
                });
            }
        };
    }
}

/// Helper: tokens to inject `state_handler`
fn state_handler_inject_tokens(crate_path: &syn::Path) -> proc_macro2::TokenStream {
    quote! {
        let state_handler = {
            use #crate_path::{StateHandlerFn, TesseraRuntime};
            |fun: Box<StateHandlerFn>| {
                TesseraRuntime::with_mut(|runtime| {
                    runtime
                        .component_tree
                        .current_node_mut()
                        .unwrap()
                        .state_handler_fn = Some(fun)
                });
            }
        };
    }
}

/// Helper: tokens to inject `on_minimize`
fn on_minimize_inject_tokens(crate_path: &syn::Path) -> proc_macro2::TokenStream {
    quote! {
        let on_minimize = {
            use #crate_path::TesseraRuntime;
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
            use #crate_path::TesseraRuntime;
            |fun: Box<dyn Fn() + Send + Sync + 'static>| {
                TesseraRuntime::with_mut(|runtime| runtime.on_close(fun));
            }
        };
    }
}

/// Helper: tokens to cleanup (pop node)
fn cleanup_tokens(crate_path: &syn::Path) -> proc_macro2::TokenStream {
    quote! {
        {
            use #crate_path::TesseraRuntime;

            TesseraRuntime::with_mut(|runtime| runtime.component_tree.pop_node());
        }
    }
}

/// Transforms a regular Rust function into a Tessera UI component.
///
/// # What It Generates
/// The macro rewrites the function body so that on every invocation (every frame in an
/// immediate‑mode pass) it:
/// 1. Registers a new component node (push) into the global `ComponentTree`
/// 2. Injects helper closures:
///    * `measure(Box<MeasureFn>)` – supply layout measuring logic
///    * `state_handler(Box<StateHandlerFn>)` – supply per‑frame interaction / event handling
///    * `on_minimize(Box<dyn Fn(bool) + Send + Sync>)` – window minimize life‑cycle hook
///    * `on_close(Box<dyn Fn() + Send + Sync>)` – window close life‑cycle hook
/// 3. Executes the original user code inside an inner closure to prevent early `return`
///    from skipping cleanup
/// 4. Pops (removes) the component node (ensuring balanced push/pop even with early return)
///
/// # Usage
///
/// Annotate a free function (no captured self) with `#[tessera]`. You may then (optionally)
/// call any of the injected helpers exactly once (last call wins if repeated).
///
/// # Example
///
/// ```rust,ignore
/// use tessera_ui::tessera;
///
/// #[tessera]
/// pub fn simple_button(label: String) {
///     // Optional layout definition
///     measure(Box::new(|_input| {
///         use tessera_ui::{ComputedData, Px};
///         Ok(ComputedData { width: Px(90), height: Px(32) })
///     }));
///
///     // Optional interaction handling
///     state_handler(Box::new(|input| {
///         // Inspect input.cursor_events / keyboard_events ...
///         let _ = input.cursor_events.len();
///     }));
///
///     on_close(Box::new(|| {
///         println!("Window closing – component had registered an on_close hook.");
///     }));
///
///     // Build children here (invoke child closures so they register themselves)
///     // child();
/// }
/// ```
///
/// # Error Handling & Early Return
///
/// Your original function body is wrapped in an inner closure; an early `return` inside
/// the body only returns from that closure, after which cleanup (node pop) still occurs.
///
/// # Parameters
///
/// * Attribute arguments are currently unused; pass nothing or `#[tessera]`.
///
/// # When NOT to Use
///
/// * For function that should not be a component.
///
/// # See Also
///
/// * [`#[shard]`](crate::shard) for navigation‑aware components with injectable shard state.
#[proc_macro_attribute]
pub fn tessera(attr: TokenStream, item: TokenStream) -> TokenStream {
    let crate_path: syn::Path = parse_crate_path(attr);

    // Parse the input function that will be transformed into a component
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident; // Function name for component identification
    let fn_vis = &input_fn.vis; // Visibility (pub, pub(crate), etc.)
    let fn_attrs = &input_fn.attrs; // Attributes like #[doc], #[allow], etc.
    let fn_sig = &input_fn.sig; // Function signature (parameters, return type)
    let fn_block = &input_fn.block; // Original function body

    // Prepare token fragments using helpers to keep function small and readable
    let register_tokens = register_node_tokens(&crate_path, fn_name);
    let measure_tokens = measure_inject_tokens(&crate_path);
    let state_tokens = state_handler_inject_tokens(&crate_path);
    let on_minimize_tokens = on_minimize_inject_tokens(&crate_path);
    let on_close_tokens = on_close_inject_tokens(&crate_path);
    let cleanup = cleanup_tokens(&crate_path);

    // Generate the transformed function with Tessera runtime integration
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            #register_tokens

            #measure_tokens

            #state_tokens

            #on_minimize_tokens

            #on_close_tokens

            // Execute the original function body within a closure to avoid early-return issues
            let result = {
                let closure = || #fn_block;
                closure()
            };

            #cleanup

            result
        }
    };

    TokenStream::from(expanded)
}

#[cfg(feature = "shard")]
/// Transforms a function into a *shard component* that can be navigated to via the routing
/// system and (optionally) provided with a lazily‑initialized per‑shard state.
///
/// # Features
/// * Generates a `StructNameDestination` (UpperCamelCase + `Destination`) implementing
///   `tessera_ui_shard::router::RouterDestination`
/// * (Optional) Injects a single `#[state]` parameter whose type:
///   - Must implement `Default + Send + Sync + 'static`
///   - Is constructed (or reused) and passed to your function body
/// * Produces a stable shard ID: `module_path!()::function_name`
///
/// # Lifecycle
/// Controlled by the generated destination (via `#[state(...)]`).
/// * Default: `Shard` – state is removed when the destination is `pop()`‑ed
/// * Override: `#[state(app)]` (or `#[state(application)]`) – persist for the entire application
///
/// When `pop()` is called and the destination lifecycle is `Shard`, the registry
/// entry is removed, freeing the state.
///
/// # Example
///
/// ```rust,ignore
/// use tessera_ui::{shard, tessera};
///
/// #[tessera]
/// #[shard]
/// fn profile_page(#[state] state: ProfileState) {
///     // Build your UI. You can navigate:
///     // router::push(OtherPageDestination { ... });
/// }
///
/// #[derive(Default)]
/// struct ProfileState {
///     // fields...
/// }
/// ```
///
/// Pushing a shard:
///
/// ```rust,ignore
/// use tessera_ui::router;
/// router::push(ProfilePageDestination { /* fields from fn params (excluding #[state]) */ });
/// ```
///
/// # Parameter Transformation
/// * At most one parameter may be annotated with `#[state]`.
/// * That parameter is removed from the *generated* function signature and supplied implicitly.
/// * All other parameters remain explicit and become public fields on the generated
///   `*Destination` struct.
///
/// # Generated Destination (Conceptual)
/// ```text
/// struct ProfilePageDestination { /* non-state params as public fields */ }
/// impl RouterDestination for ProfilePageDestination {
///     fn exec_component(&self) { profile_page(/* fields */); }
///     fn shard_id(&self) -> &'static str { "<module>::profile_page" }
/// }
/// ```
///
/// # Limitations
/// * No support for multiple `#[state]` params (compile panic if violated)
/// * Do not manually implement `RouterDestination` for these pages; rely on generation
///
/// # See Also
/// * Routing helpers: `tessera_ui::router::{push, pop, router_root}`
/// * Shard state registry: `tessera_ui_shard::ShardRegistry`
///
/// # Safety
/// Internally uses an unsafe cast inside the registry to recover `Arc<T>` from
/// `Arc<dyn ShardState>`; this is encapsulated and not exposed.
///
/// # Errors / Panics
/// * Panics at compile time if multiple `#[state]` parameters are used or unsupported
///   pattern forms are encountered.
#[proc_macro_attribute]
pub fn shard(attr: TokenStream, input: TokenStream) -> TokenStream {
    use heck::ToUpperCamelCase;
    use syn::Pat;

    let crate_path: syn::Path = if attr.is_empty() {
        syn::parse_quote!(::tessera_ui)
    } else {
        syn::parse(attr).expect("Expected a valid path like `crate` or `tessera_ui`")
    };

    // 1. Parse the function marked by the macro
    let mut func = parse_macro_input!(input as ItemFn);

    // 2. Handle #[state] parameters, ensuring it's unique and removing it from the signature
    //    Also parse optional lifecycle argument: #[state(app)] or #[state(shard)]
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

    // 3. Extract the name and type of the state parameter
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

    // 4. Save the original function body and function name
    let func_body = func.block;
    let func_name_str = func.sig.ident.to_string();

    // 5. Get the remaining function attributes and the modified signature
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

    // 6. Use quote! to generate the new TokenStream code
    //    Prepare optional lifecycle override method for RouterDestination impl.
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
                // Generate a RouterDestination struct for the function
                #func_vis struct #struct_name {
                    #(#dest_fields),*
                }

                // Implement the RouterDestination trait for the generated struct
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

                // Rebuild the function, keeping its attributes and visibility, but using the modified signature
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
                // Generate a RouterDestination struct for the function
                #func_vis struct #struct_name {
                    #(#dest_fields),*
                }

                // Implement the RouterDestination trait for the generated struct
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

                // Rebuild the function, keeping its attributes and visibility, but using the modified signature
                #(#func_attrs)*
                #func_vis #func_sig_modified {
                    #func_body
                }
            }
        }
    };

    // 7. Return the generated code as a TokenStream
    TokenStream::from(expanded)
}
