use super::lrfrp_ir::LrfrpIR;
use proc_macro::TokenStream;
use quote::quote;

pub fn codegen(lrfrp_ir: LrfrpIR) -> TokenStream {
    let LrfrpIR {
        module,
        input,
        output,
        args,
        declarations,
        body,
    } = lrfrp_ir;

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
            #(#declarations)*

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
                #[inline]
                pub fn new(#args_field) -> Self {
                    FRP {
                        running: false,
                        output: Out::default(),
                        #args_initialization
                        cell: Cell::default(),
                    }.cell_initializations()
                }

                #[inline]
                fn cell_initializations(mut self) -> Self {
                    #cell_initializations
                    self
                }

                #[inline]
                pub fn sample(&self) -> core::option::Option<&Out> {
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

    #[cfg(feature = "print-codegen")]
    print_codegen(&token_stream);

    token_stream.into()
}

#[cfg(feature = "print-codegen")]
fn print_codegen(token_stream: &proc_macro2::TokenStream) {
    use ::rustfmt::config::Config;
    use ::rustfmt::Input;
    use std::io;

    let input = Input::Text(token_stream.to_string());
    let mut output = io::stderr();
    let res = rustfmt::format_input(input, &Config::default(), Some(&mut output));
    if let Ok((_, file_map, _)) = res {
        eprintln!(r#"{}"#, file_map[0].1);
    } else {
        eprintln!("formatting generated codes failed");
    }
}
