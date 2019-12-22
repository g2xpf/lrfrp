use super::lfrp_ir::LfrpIR;
use proc_macro::TokenStream;
use quote::quote;

pub fn codegen(lfrp_ir: LfrpIR) -> TokenStream {
    let LfrpIR {
        module,
        input,
        output,
        args,
        body,
    } = lfrp_ir;

    let module_name = &module.name;

    let args_field = args.as_ref().map(|_| {
        quote! {
            args: Args,
        }
    });

    let args_initialization = args.as_ref().map(|_| {
        quote! {
            args,
        }
    });

    let cell_definition = body.cell_definition();
    let calculations = &body.dependencies;

    let cell_initializations = body.cell_initializations();
    let cell_updates = body.cell_updates();

    let token_stream = quote! {
        #[allow(non_snake_case)]
        mod #module_name {
            #input
            #output
            #args
            #cell_definition

            #[derive(Clone, Default)]
            pub struct FRP {
                running: bool,
                output: Out,
                #args_field
                cell: Cell,
            }

            impl FRP {
                pub fn new(#args_field) -> Self {
                    FRP {
                        running: false,
                        output: Out::default(),
                        #args_initialization
                        cell: Cell::default(),
                    }.cell_initializations()
                }

                fn cell_initializations(mut self) -> Self {
                    #cell_initializations
                    self
                }

                pub fn sample(&self) -> Option<&Out> {
                    if self.running {
                        Some(&self.output)
                    } else {
                        None
                    }
                }

                pub fn run(&mut self, input: &In) {
                    self.running |= true;
                    #(#calculations)*
                    #cell_updates
                }
            }
        }
    };

    token_stream.into()
}
