use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub(crate) fn auto_command_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    let expanded = quote! {
        #[tauri::command]
        #input_fn

        inventory::submit! {
            crate::AutoCommand {
                name: #fn_name_str,
                handler: || {
                    let handler: fn(tauri::ipc::Invoke) -> bool = tauri::generate_handler![#fn_name];
                    handler
                }
            }
        }
    };

    TokenStream::from(expanded)
}