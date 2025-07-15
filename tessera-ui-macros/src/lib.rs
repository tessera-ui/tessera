//! # Tessera Macros
//!
//! This crate provides procedural macros for the Tessera UI framework.
//! The main export is the `#[tessera]` attribute macro, which transforms
//! regular Rust functions into Tessera UI components.
//!
//! ## Usage
//!
//! ```rust
//! use tessera_ui_macros::tessera;
//!
//! #[tessera]
//! fn my_component() {
//!     // Component logic here
//!     // The macro provides access to `measure` and `state_handler` functions
//! }
//! ```
//!
//! The `#[tessera]` macro automatically:
//! - Registers the function as a component in the Tessera component tree
//! - Injects `measure` and `state_handler` functions into the component scope
//! - Handles component tree management (adding/removing nodes)
//! - Provides error safety by wrapping the function body

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// The `#[tessera]` attribute macro transforms a regular Rust function into a Tessera UI component.
///
/// This macro performs several key transformations:
/// 1. Registers the function as a node in the Tessera component tree
/// 2. Injects `measure` and `state_handler` functions into the component scope
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
/// - Accesses the Tessera runtime to manage the component tree
/// - Creates a new component node with the function name
/// - Provides closures for `measure` and `state_handler` functionality
/// - Executes the original function body within a safe closure
/// - Cleans up the component tree after execution
///
/// ## Example
///
/// ```rust
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

                TesseraRuntime::write()
                    .component_tree
                    .add_node(
                        ComponentNode {
                            fn_name: stringify!(#fn_name).to_string(),
                            measure_fn: None,
                            state_handler_fn: None,
                        }
                    );
            }

            // Step 2: Inject the `measure` function into the component scope
            // This allows components to define custom layout behavior
            let measure = {
                use tessera_ui::{MeasureFn, TesseraRuntime};
                |fun: Box<MeasureFn>| {
                    TesseraRuntime::write()
                        .component_tree
                        .current_node_mut()
                        .unwrap()
                        .measure_fn = Some(fun);
                }
            };

            // Step 3: Inject the `state_handler` function into the component scope
            // This allows components to handle user interactions and events
            let state_handler = {
                use tessera_ui::{StateHandlerFn, TesseraRuntime};
                |fun: Box<StateHandlerFn>| {
                    TesseraRuntime::write()
                        .component_tree
                        .current_node_mut()
                        .unwrap()
                        .state_handler_fn = Some(fun);
                }
            };

            // Step 4: Execute the original function body within a closure
            // This prevents early returns from breaking the component tree structure
            let result = {
                let closure = || #fn_block;
                closure()
            };

            // Step 5: Clean up the component tree by removing this node
            // This ensures proper tree management and prevents memory leaks
            {
                use tessera_ui::TesseraRuntime;

                TesseraRuntime::write()
                    .component_tree
                    .pop_node();
            }

            result
        }
    };

    TokenStream::from(expanded)
}
