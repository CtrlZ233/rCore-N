
mod syscall;

extern crate alloc;
extern crate proc_macro;
use alloc::vec::Vec;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use syscall::Arguments;
use regex::Regex;

/// 生成系统调用用户态的接口
#[proc_macro_derive(GenSysMacro, attributes(arguments))]
pub fn syscall_macro_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    // ident 当前枚举名称
    let DeriveInput { ident, .. } = input;
    let mut comment_arms = Vec::new();
    if let syn::Data::Enum(syn::DataEnum { variants, .. }) = input.data {
        for variant in variants {
            // 当前枚举项名称如 Alex, Box
            let ident_item = &variant.ident;
            let re = Regex::new(r"(?P<up>[A-Z])").unwrap();
            let before = ident_item.to_string();
            let mut fn_name_str = re.replace_all(before.as_str(), r"_$up").to_lowercase();
            fn_name_str.insert_str(0, "sys");
            // println!("{}", fn_name_str);
            let ident_name = syn::Ident::new(fn_name_str.as_str(), Span::call_site());
            if let Ok(args) = Arguments::from_attributes(&variant.attrs) {
                // 获取属性中定义的参数信息
                let args_vec: Vec<syn::Ident> = args.args.unwrap().value().split(", ").map(|s| syn::Ident::new(s, Span::call_site())).collect();
                let len = args_vec.len();
                let syscall_fn = quote::format_ident!("syscall{}", len);
                let mut doc = String::from("参数类型为 ");
                for  idx in 0..(len - 1) {
                    doc.push_str(&args_vec[idx].to_string().as_str());
                    doc.push_str(": usize, ");
                }
                doc.push_str(&args_vec[len - 1].to_string().as_str());
                doc.push_str(": usize");
                // 生成对应的系统调用函数
                comment_arms.push(quote! (
                    #[doc = #doc]
                    #[inline]
                    pub fn #ident_name(#(#args_vec: usize), *) -> isize {
                        unsafe {
                            #syscall_fn(#ident::#ident_item as usize, #(#args_vec),*)
                        }
                    }
                ));
            } else {
                comment_arms.push(quote! ( 
                    #[inline]
                    pub fn #ident_name() -> isize {
                        unsafe {
                            syscall0(#ident::#ident_item as usize)
                        }
                    }
                ));
            }
            
        }
    }
    quote!(
        #(#comment_arms)*
        macro_rules! syscall {
            ($($name:ident($a:ident, $($b:ident, $($c:ident, $($d:ident, $($e:ident, $($f:ident, $($g:ident)?)?)?)?)?)?);)+) => {
                $(
                    #[inline]
                    pub unsafe fn $name($a: usize, $($b: usize, $($c: usize, $($d: usize, $($e: usize, $($f: usize, $($g: usize)?)?)?)?)?)?) -> isize {
                        let ret: isize;
                        core::arch::asm!(
                            "ecall",
                            in("a7") $a,
                            $(
                                in("a0") $b,
                                $(
                                    in("a1") $c,
                                    $(
                                        in("a2") $d,
                                        $(
                                            in("a3") $e,
                                            $(
                                                in("a4") $f,
                                                $(
                                                    in("a5") $g,
                                                )?
                                            )?
                                        )?
                                    )?
                                )?
                            )?
                            lateout("a0") ret,
                            options(nostack),
                        );
                        ret
                    }
                )+
            };
        }
        
        syscall! {
            syscall0(a,);
            syscall1(a, b,);
            syscall2(a, b, c,);
            syscall3(a, b, c, d,);
            syscall4(a, b, c, d, e,);
            syscall5(a, b, c, d, e, f,);
            syscall6(a, b, c, d, e, f, g);
        }
    ).into()
}


/// 生成系统调用内核的 trait
#[proc_macro_derive(GenSysTrait, attributes(arguments))]
pub fn syscall_trait_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    // ident 当前枚举名称
    let mut trait_fns = Vec::new();
    if let syn::Data::Enum(syn::DataEnum { variants, .. }) = input.data {
        for variant in variants {
            // 当前枚举项名称如 Alex, Box
            let ident_item = &variant.ident;
            let re = Regex::new(r"(?P<up>[A-Z])").unwrap();
            let before = ident_item.to_string();
            let mut fn_name_str = re.replace_all(before.as_str(), r"_$up").to_lowercase();
            fn_name_str.insert_str(0, "sys");
            // println!("{}", fn_name_str);
            let ident_name = syn::Ident::new(fn_name_str.as_str(), Span::call_site());
            if let Ok(args) = Arguments::from_attributes(&variant.attrs) {
                // 获取属性中定义的参数信息
                let args_vec: Vec<syn::Ident> = args.args.unwrap().value().split(", ").map(|s| syn::Ident::new(s, Span::call_site())).collect();
                // 生成 SyscallTrait 中对应的 成员方法代码
                trait_fns.push(quote!(
                    #[inline]
                    fn #ident_name(&self, #(#args_vec: usize), *) -> isize {
                        unimplemented!()
                    }
                ));
            } else {
                trait_fns.push(quote!(
                    #[inline]
                    fn #ident_name(&self) -> isize {
                        unimplemented!()
                    }
                ));
            }
        }
    }
    quote!(
        pub trait SyscallTrait: Sync {
            #(#trait_fns)*
        }
    ).into()
}