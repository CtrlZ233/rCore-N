#![allow(unused_assignments)]

extern crate alloc;
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn};

/// 根据 feature 生成 vdso 接口
#[proc_macro_attribute]
pub fn vdso(attr: TokenStream, item: TokenStream) -> TokenStream {
    // println!("attr: \"{}\"", attr.to_string());
    let mut base_fn = TokenStream::from(item.clone());
    // println!("item: {}", base_fn);
    let feature = attr.to_string();
    let input_fn = parse_macro_input!(item as ItemFn);
    let ident = input_fn.sig.ident;
    let args = input_fn.sig.inputs.to_token_stream();
    let output = input_fn.sig.output.to_token_stream();
    let mut derive_fn = TokenStream::default();
    let ident_name = quote::format_ident!("{}_so", ident);
    if output.is_empty() {
        derive_fn = quote!(
            #[cfg(feature = #feature)]
            #[no_mangle]
            #[inline(never)]
            fn #ident_name(#args) {
                println!("hello");
            }
        ).into();
    } else {
        derive_fn = quote!(
            #[cfg(feature = #feature)]
            #[no_mangle]
            #[inline(never)]
            fn #ident_name(#args) -> #output {
                println!("hello");
            }
        ).into();
    }
    // println!("{}", derive_fn.to_string());
    base_fn.extend(derive_fn);
    // println!("{}", base_fn.to_string());
    base_fn
    
}

// #[proc_macro_attribute(Vdso, attributes(feature))]
// pub fn vdso_macro_derive(input: TokenStream) -> TokenStream {
//     let input: DeriveInput = parse_macro_input!(input);
//     // ident 当前枚举名称
//     let DeriveInput { ident, .. } = input;
//     let mut comment_arms = Vec::new();
//     if let syn::Data::Enum(syn::DataEnum { variants, .. }) = input.data {
//         for variant in variants {
//             // 当前枚举项名称如 Alex, Box
//             let ident_item = &variant.ident;
//             let re = Regex::new(r"(?P<up>[A-Z])").unwrap();
//             let before = ident_item.to_string();
//             let mut fn_name_str = re.replace_all(before.as_str(), r"_$up").to_lowercase();
//             fn_name_str.insert_str(0, "sys");
//             println!("{}", fn_name_str);
//             let ident_name = syn::Ident::new(fn_name_str.as_str(), Span::call_site());
//             if let Ok(args) = Arguments::from_attributes(&variant.attrs) {
//                 // 获取属性中定义的参数信息
//                 let args_vec: Vec<syn::Ident> = args.args.unwrap().value().split(", ").map(|s| syn::Ident::new(s, Span::call_site())).collect();
//                 let len = args_vec.len();
//                 let syscall_fn = quote::format_ident!("syscall{}", len);
//                 let mut doc = String::from("参数类型为 ");
//                 for  idx in 0..(len - 1) {
//                     doc.push_str(&args_vec[idx].to_string().as_str());
//                     doc.push_str(": usize, ");
//                 }
//                 doc.push_str(&args_vec[len - 1].to_string().as_str());
//                 doc.push_str(": usize");
//                 // 生成对应的系统调用函数
//                 comment_arms.push(quote! (
//                     #[doc = #doc]
//                     #[inline]
//                     pub fn #ident_name(#(#args_vec: usize), *) -> isize {
//                         unsafe {
//                             #syscall_fn(#ident::#ident_item as usize, #(#args_vec),*)
//                         }
//                     }
//                 ));
//             } else {
//                 comment_arms.push(quote! ( 
//                     #[inline]
//                     pub fn #ident_name() -> isize {
//                         unsafe {
//                             syscall0(#ident::#ident_item as usize)
//                         }
//                     }
//                 ));
//             }
            
//         }
//     }
//     quote!().into()
// }