//! # Tessera Macros
//!
//! This crate provides procedural macros for the Tessera UI framework.
//! The main export is the `#[tessera]` attribute macro, which transforms
//! regular Rust functions into Tessera UI components.
//!
//! ## Usage
//!
//! ```
//! use tessera_ui_macros::tessera;
//!
//! #[tessera]
//! fn my_component() {
//!     // Component logic here
//!     // The macro provides access to `measure`, `state_handler` and `on_minimize` functions
//! }
//! ```
//!
//! The `#[tessera]` macro automatically:
//! - Registers the function as a component in the Tessera component tree
//! - Injects `measure`, `state_handler` and `on_minimize` functions into the component scope
//! - Handles component tree management (adding/removing nodes)
//! - Provides error safety by wrapping the function body

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// The `#[tessera]` attribute macro transforms a regular Rust function into a Tessera UI component.
///
/// This macro performs several key transformations:
/// 1. Registers the function as a node in the Tessera component tree
/// 2. Injects `measure`, `state_handler` and `on_minimize` functions into the component scope
/// 3. Manages component tree lifecycle (push/pop operations)
/// 4. Provides error safety by wrapping the original function body
///
/// ## Parameters
///
/// - `_attr`: Attribute arguments (currently unused)
/// - `item`: The function to be transformed into a component
///
/// ## Generated Code
///
/// The macro generates code that:
///
/// - Accesses the Tessera runtime to manage the component tree
/// - Creates a new component node with the function name
/// - Provides closures for `measure` and `state_handler` functionality
/// - Executes the original function body within a safe closure
/// - Cleans up the component tree after execution
///
/// ## Example
///
/// ```rust,ignore
/// use tessera_ui_macros::tessera;
///
/// #[tessera]
/// fn button_component(label: String) {
///     // The macro provides access to these functions:
///     measure(Box::new(|_| {
///         // Custom layout logic
///         use tessera_ui::{ComputedData, Px};
///         Ok(ComputedData {
///             width: Px(100),
///             height: Px(50),
///         })
///     }));
///     
///     state_handler(Box::new(|_| {
///         // Event handling logic
///     }));
///
///     on_minimize(Box::new(|minimized| {
///         if minimized {
///             println!("Window minimized!");
///         } else {
///             println!("Window restored!");
///         }
///     }));
/// }
/// ```
///
/// ## Error Handling
///
/// The macro wraps the original function body in a closure to prevent
/// early returns from breaking the component tree structure. This ensures
/// that the component tree is always properly cleaned up, even if the
/// component function returns early.
#[proc_macro_attribute]
pub fn tessera(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function that will be transformed into a component
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident; // Function name for component identification
    let fn_vis = &input_fn.vis; // Visibility (pub, pub(crate), etc.)
    let fn_attrs = &input_fn.attrs; // Attributes like #[doc], #[allow], etc.
    let fn_sig = &input_fn.sig; // Function signature (parameters, return type)
    let fn_block = &input_fn.block; // Original function body

    // Generate the transformed function with Tessera runtime integration
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            // Step 1: Register this function as a component node in the tree
            {
                use tessera_ui::{TesseraRuntime, ComponentNode};

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

            // Step 2: Inject the `measure` function into the component scope
            // This allows components to define custom layout behavior
            let measure = {
                use tessera_ui::{MeasureFn, TesseraRuntime};
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

            // Step 3: Inject the `state_handler` function into the component scope
            // This allows components to handle user interactions and events
            let state_handler = {
                use tessera_ui::{StateHandlerFn, TesseraRuntime};
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

            // Step 4: Inject the `on_minimize` function into the component scope
            // This allows components to respond to window minimize events
            let on_minimize = {
                use tessera_ui::TesseraRuntime;
                |fun: Box<dyn Fn(bool) + Send + Sync + 'static>| {
                    TesseraRuntime::with_mut(|runtime| runtime.on_minimize(fun));
                }
            };

            // Step 4b: Inject the `on_close` function into the component scope
            // This allows components to respond to window close events
            let on_close = {
                use tessera_ui::TesseraRuntime;
                |fun: Box<dyn Fn() + Send + Sync + 'static>| {
                    TesseraRuntime::with_mut(|runtime| runtime.on_close(fun));
                }
            };

            // Step 5: Execute the original function body within a closure
            // This prevents early returns from breaking the component tree structure
            let result = {
                let closure = || #fn_block;
                closure()
            };

            // Step 6: Clean up the component tree by removing this node
            // This ensures proper tree management and prevents memory leaks
            {
                use tessera_ui::TesseraRuntime;

                TesseraRuntime::with_mut(|runtime| runtime.component_tree.pop_node());
            }

            result
        }
    };

    TokenStream::from(expanded)
}

#[cfg(feature = "shard")]
#[proc_macro_attribute]
pub fn shard(_args: TokenStream, input: TokenStream) -> TokenStream {
    use syn::Pat;

    // 1. Parse the function marked by the macro
    let mut func = parse_macro_input!(input as ItemFn);

    // 2. Find and remove the state parameter from the function signature (assumed to be the first parameter)
    let state_param = match func.sig.inputs.iter().next() {
        Some(syn::FnArg::Typed(pat_type)) => pat_type.clone(),
        _ => panic!("#[shard] function must have at least one typed parameter for the state."),
    };
    // Remove the first parameter
    func.sig.inputs = func.sig.inputs.iter().skip(1).cloned().collect();

    // 3. Extract the name and type of the state parameter
    let state_name = match *state_param.pat {
        Pat::Ident(pat_ident) => pat_ident.ident,
        _ => panic!(
            "Unsupported parameter pattern in #[shard] function. Please use a simple identifier like `state`."
        ),
    };
    let state_type = state_param.ty;

    // 4. Save the original function body and function name
    let func_body = func.block;
    let func_name_str = func.sig.ident.to_string();

    // 5. Get the remaining function attributes and the modified signature
    let func_attrs = &func.attrs;
    let func_vis = &func.vis;
    let func_sig_modified = &func.sig;

    // 6. Use quote! to generate the new TokenStream code
    let expanded = quote! {
        // Rebuild the function, keeping its attributes and visibility, but using the modified signature
        #(#func_attrs)*
        #func_vis #func_sig_modified {
            // Generate a stable unique ID at the call site
            const SHARD_ID: &str = concat!(module_path!(), "::", #func_name_str);

            // Call the global registry and pass the original function body as a closure
            // The state parameter is reintroduced here as the closure's parameter
            // The use of unsafe here is because its implementation is very evil 😈, do not attempt to call it manually elsewhere
            unsafe {
                ::tessera_ui::tessera_ui_shard::ShardRegistry::get().init_or_get::<#state_type, _, _>(
                    SHARD_ID,
                    |#state_name| {
                        #func_body
                    },
                )
            }
        }
    };

    // 7. Return the generated code as a TokenStream
    TokenStream::from(expanded)
}
