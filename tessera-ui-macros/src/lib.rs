//! # Tessera Macros
//!
//! This crate provides procedural macros for the Tessera UI framework.
//! The main export is the `#[tessera]` attribute macro, which transforms
//! regular Rust functions into Tessera UI components.
//!
//! ## Usage
//!
//! ```rust,ignore
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
    let register_tokens = register_node_tokens(&crate_path, &fn_name);
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
#[proc_macro_attribute]
pub fn shard(attr: TokenStream, input: TokenStream) -> TokenStream {
    use heck::ToUpperCamelCase;
    use syn::Pat;

    let crate_path: syn::Path = if attr.is_empty() {
        // Default to `tessera_ui` if no path is provided
        syn::parse_quote!(::tessera_ui)
    } else {
        // Parse the provided path, e.g., `crate` or `tessera_ui`
        syn::parse(attr).expect("Expected a valid path like `crate` or `tessera_ui`")
    };

    // 1. Parse the function marked by the macro
    let mut func = parse_macro_input!(input as ItemFn);

    // 2. Handle #[state] and #[route_controller] parameters, ensuring they are unique and removing them from the signature
    let mut state_param = None;
    let mut controller_param = None;
    let mut new_inputs = syn::punctuated::Punctuated::new();
    for arg in func.sig.inputs.iter() {
        if let syn::FnArg::Typed(pat_type) = arg {
            let is_state = pat_type
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("state"));
            let is_controller = pat_type
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("route_controller"));
            if is_state {
                if state_param.is_some() {
                    panic!(
                        "#[shard] function must have at most one parameter marked with #[state]."
                    );
                }
                state_param = Some(pat_type.clone());
                continue;
            }
            if is_controller {
                if controller_param.is_some() {
                    panic!(
                        "#[shard] function must have at most one parameter marked with #[route_controller]."
                    );
                }
                controller_param = Some(pat_type.clone());
                continue;
            }
        }
        new_inputs.push(arg.clone());
    }
    func.sig.inputs = new_inputs;

    // 3. Extract the name and type of the state/controller parameters
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
    let (controller_name, controller_type) = if let Some(controller_param) = controller_param {
        let name = match *controller_param.pat {
            Pat::Ident(ref pat_ident) => pat_ident.ident.clone(),
            _ => panic!(
                "Unsupported parameter pattern in #[shard] function. Please use a simple identifier like `ctrl`."
            ),
        };
        (Some(name), Some(controller_param.ty))
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

    // Only keep the parameters that are not marked with #[state] or #[route_controller]
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
    let expanded = {
        // `exec_component` only passes struct fields (unmarked parameters).
        let exec_args = param_idents
            .iter()
            .map(|ident| quote! { self.#ident.clone() });

        if let Some(state_type) = state_type {
            let state_name = state_name.as_ref().unwrap();
            let controller_inject = if let Some((ref ctrl_name, ref ctrl_ty)) =
                controller_name.zip(controller_type.as_ref())
            {
                quote! {
                    // Inject RouteController instance here
                    let #ctrl_name = #ctrl_ty::new();
                }
            } else {
                quote! {}
            };
            quote! {
                // Generate a RouterDestination struct for the function
                /// This struct represents a route destination for the #[shard] function
                ///
                /// # Example
                ///
                /// ```ignore
                /// controller.push(AboutPageDestination {
                ///     title: "About".to_string(),
                ///     description: "This is the about page.".to_string(),
                /// })
                /// ```
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
                }

                // Rebuild the function, keeping its attributes and visibility, but using the modified signature
                #(#func_attrs)*
                #func_vis #func_sig_modified {
                    // Generate a stable unique ID at the call site
                    const SHARD_ID: &str = concat!(module_path!(), "::", #func_name_str);

                    // Call the global registry and pass the original function body as a closure
                    // Inject state/controller here
                    unsafe {
                        #crate_path::tessera_ui_shard::ShardRegistry::get().init_or_get::<#state_type, _, _>(
                            SHARD_ID,
                            |#state_name| {
                                #controller_inject
                                #func_body
                            },
                        )
                    }
                }
            }
        } else {
            let controller_inject = if let Some((ref ctrl_name, ref ctrl_ty)) =
                controller_name.zip(controller_type.as_ref())
            {
                quote! {
                    // Inject RouteController instance here
                    let #ctrl_name = #ctrl_ty::new();
                }
            } else {
                quote! {}
            };
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
                }

                // Rebuild the function, keeping its attributes and visibility, but using the modified signature
                #(#func_attrs)*
                #func_vis #func_sig_modified {
                    #controller_inject
                    #func_body
                }
            }
        }
    };

    // 7. Return the generated code as a TokenStream
    TokenStream::from(expanded)
}
