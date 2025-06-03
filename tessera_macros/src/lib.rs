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
                use tessera::{TesseraRuntime, ComponentNode, Constraint}; // Added Constraint

                TesseraRuntime::write()
                    .component_tree
                    .add_node(
                        ComponentNode {
                            measure_fn: None,
                        },
                        Constraint::NONE // Pass Constraint::NONE as the intrinsic_constraint
                                         // The component's measure_fn will use its args
                                         // to define its behavior and merge with parent constraint.
                    );
            }



            let measure = {
                use tessera::{BasicDrawable, ComponentNode, MeasureFn, TesseraRuntime};
                |fun: Box<MeasureFn>| {
                    TesseraRuntime::write()
                        .component_tree
                        .current_node_mut()
                        .unwrap()
                        .measure_fn = Some(fun);
                }
            };

            {
                #fn_block
            }

            {
                use tessera::TesseraRuntime;

                TesseraRuntime::write()
                    .component_tree
                    .pop_node();
            }
        }
    };

    TokenStream::from(expanded)
}
