use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn tessera(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse input component function
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;

    // Insert runtime access codes
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            {
                use tessera::{TesseraRuntime, ComponentNode};

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



            let measure = {
                use tessera::{MeasureFn, TesseraRuntime};
                |fun: Box<MeasureFn>| {
                    TesseraRuntime::write()
                        .component_tree
                        .current_node_mut()
                        .unwrap()
                        .measure_fn = Some(fun);
                }
            };

            let state_handler = {
                use tessera::{StateHandlerFn, TesseraRuntime};
                |fun: Box<StateHandlerFn>| {
                    TesseraRuntime::write()
                        .component_tree
                        .current_node_mut()
                        .unwrap()
                        .state_handler_fn = Some(fun);
                }
            };

            // Package the function body into a closure
            // so early return won't break the component tree
            let result = {
                let closure = || #fn_block;
                closure()
            };

            {
                use tessera::TesseraRuntime;

                TesseraRuntime::write()
                    .component_tree
                    .pop_node();
            }

            result
        }
    };

    TokenStream::from(expanded)
}
