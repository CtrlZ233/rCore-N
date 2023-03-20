#![allow(unused_assignments)]

extern crate alloc;
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use proc_macro2::Span;
use syn::{parse_macro_input, ItemFn, Ident};
use regex::Regex;

#[proc_macro]
pub fn get_libfn(item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    // let attrs: Vec<_> = input_fn.attrs.into_iter().map(|attr| attr.parse_meta().ok()).collect();
    let ident = input_fn.sig.ident;
    let args = input_fn.sig.inputs.to_token_stream();
    let mut args_str = args.to_string();
    args_str.push(',');
    let re = Regex::new(r"(?s):[^,]*,").unwrap();
    let mut args_type_str = re.replace_all(args_str.as_str(), r",").to_string();
    args_type_str.push(' ');
    let mut args_value: Vec<_> = args_type_str.split(" , ").collect();
    args_value.pop();
    // println!("{:?}", args_value);
    let args_value: Vec<syn::Ident> = args_value.iter().map(|s| Ident::new(*s, Span::call_site())).collect();
    // println!("{:?}", args_value[0]);
    let mut output = input_fn.sig.output.to_token_stream();
    let mut derive_fn = TokenStream::default();
    let vdso_name = format!(".vdso.{}", ident.to_string());
    let vdso_ptr = Ident::new(ident.to_string().to_uppercase().as_str(), Span::call_site());
    let vdso_ptr = quote::format_ident!("VDSO_{}", vdso_ptr);
    if output.is_empty() {
        output.extend(quote!(-> ()));
    }
    let init_fn = quote::format_ident!("init_{}", ident);
    derive_fn = quote!(
        #[no_mangle]
        #[link_section = #vdso_name]
        pub static mut #vdso_ptr: usize = 0;
        #[cfg(feature = "kernel")]
        pub fn #init_fn(ptr: usize) {
            unsafe { #vdso_ptr = ptr; }
        }
        #[inline(never)]
        pub fn #ident(#args) #output {
            unsafe {
                let func: fn(#args) #output = core::mem::transmute(#vdso_ptr);
                func(#(#args_value),*)
            }
        }
    ).into();
    // println!("{}", derive_fn.to_string());
    derive_fn
}
