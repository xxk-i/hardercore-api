extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(Stats, attributes(update))]
pub fn derive_stats_fn(item: TokenStream) -> TokenStream {
    println!("item: \"{}\"", item.to_string());
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn stat(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());
    item
}