use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, PatType, Type};

fn formatted_arg_error_msg(arg_name: &str, arg_pos: usize, fn_name: &str) -> String {
    format!(
        "Expected argument {} ('{}') to be i64, for {}",
        arg_pos, arg_name, fn_name
    )
}

#[proc_macro_attribute]
pub fn quantinxis_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_args = &input.sig.inputs;
    let fn_body = &input.block;
    let fn_output = &input.sig.output;

    let mut arg_extractions = Vec::new();
    let mut arg_names = Vec::new();

    for (i, arg) in fn_args.iter().enumerate() {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            let arg_name = match **pat {
                syn::Pat::Ident(ref ident) => &ident.ident,
                _ => panic!("Unsupported pattern"),
            };

            let err_msg = formatted_arg_error_msg(&arg_name.to_string(), i, &fn_name.to_string());

            let extract_code = match **ty {
                Type::Path(ref type_path) => {
                    let type_ident = &type_path.path.segments.last().unwrap().ident;
                    match type_ident.to_string().as_str() {
                        "i64" => quote! {
                            let #arg_name = match args[#i] {
                                // Value::Int(n) => n,
                                Value::Number(n) => n as i64,
                                Value::Boolean(n) => n as i64,
                                _ => return Err(#err_msg.to_string()),
                            };
                        },
                        "f64" => quote! {
                            let #arg_name = match args[#i] {
                                // Value::Int(n) => n as f64,
                                Value::Number(n) => n,
                                Value::Boolean(n) => n as i64 as f64,
                                _ => return Err(#err_msg.to_string()),
                            };
                        },
                        "bool" => quote! {
                            let #arg_name = match args[#i] {
                                // Value::Int(n) => n as bool,
                                Value::Number(n) => n as i64 as bool,
                                Value::Boolean(b) => b,
                                _ => return Err(#err_msg.to_string()),
                            };
                        },
                        "str" => quote! {
                            let #arg_name = match args[#i] {
                                Value::Str(s) => s,
                                _ => return Err(#err_msg.to_string()),
                            };
                        },
                        "Vec" => quote! {
                            let #arg_name = match &args[#i] {
                                Value::Array(arr) => arr.clone(),
                                _ => return Err(#err_msg.to_string()),
                            };
                        },
                        "HashMap<String, Value>" => quote! {
                            let #arg_name = match &args[#i] {
                                Value::Map(map) => map.clone(),
                                _ => return Err(#err_msg.to_string()),
                            };
                        },
                        _ => panic!("Unsupported type {}", type_ident),
                    }
                }
                _ => panic!("Unsupported argument type"),
            };

            arg_extractions.push(extract_code);
            arg_names.push(arg_name.clone());
        }
    }

    let args_len = arg_names.len();
    let expanded = quote! {
        pub fn #fn_name(args: &[Value]) #fn_output {
            if args.len() != #args_len {
                return Err(format!("Expected {} arguments, but got {}", #args_len, args.len()));
            }

            #(#arg_extractions)*

            #fn_body
        }
    };

    TokenStream::from(expanded)
}
