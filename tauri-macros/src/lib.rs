mod auto_command_macro;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn auto_command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    auto_command_macro::auto_command_macro( _attr,item)
}