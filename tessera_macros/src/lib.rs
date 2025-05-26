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
            use tessera::{TesseraRuntime, ComponentNode, DEFAULT_LAYOUT_DESC, Constraint};

            {
                TesseraRuntime::write().component_tree.add_node(ComponentNode {
                    layout_desc: Box::new(DEFAULT_LAYOUT_DESC),
                    constraint: Constraint::NONE,
                    drawable: None,
                });
            }
            {
                #fn_block
            }
            {
                TesseraRuntime::write().component_tree.pop_node();
            }
        }
    };

    TokenStream::from(expanded)
}
