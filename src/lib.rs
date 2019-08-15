extern crate proc_macro; // in 2018 edition, proc-macro still needs use extern crate 
use proc_macro::TokenStream;
use syn::{ parse_macro_input, DeriveInput, NestedMeta, Meta, Data, Fields, ItemFn, FnArg, AttributeArgs, DataStruct };
use quote::quote;
use proc_macro2::{Ident, Span};


#[proc_macro]
pub fn my_proc_macro(ident: TokenStream) -> TokenStream {
    let new_func_name = format!("test_{}", ident.to_string());
    let concated_ident = Ident::new(&new_func_name, Span::call_site()); // 创建新的ident，函数名

    let expanded = quote! {
        // 不能直接这样写trait bound，T: Debug
        // 会报错，找不到Debug trait，最好给出full path
        fn #concated_ident<T: std::fmt::Debug>(t: T) {
            println!("{:?}", t);
        }
    };
    expanded.into()
}


// derive proc-macro
#[proc_macro_derive(Show)]
pub fn derive_show(item: TokenStream) -> TokenStream {
    // 解析整个token tree
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident; // 结构体名字

    // 提取结构体里的字段
    let expanded = match input.data {
        Data::Struct(DataStruct{ref fields,..}) => {
            if let Fields::Named(ref fields_name) = fields {
                // 结构体中可能是多个字段
                let get_selfs: Vec<_> = fields_name.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap(); // 字段名字
                    quote! {
                        &self.#field_name
                    }
                }).collect();

            let implemented_show = quote! {
                // 下面就是Display trait的定义了
                // use std::fmt; // 不要这样import，因为std::fmt是全局的，无法做到卫生性(hygiene)
                // 编译器会报错重复import fmt当你多次使用Show之后
                impl std::fmt::Display for #struct_name {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        // #(#get_self),*，这是多重匹配，生成的样子大概是这样：&self.a, &self.b, &self.c, ...
                        // 用法和标准宏有点像，关于多个匹配，可以看这个文档
                        // https://docs.rs/quote/1.0.0/quote/macro.quote.html
                        write!(f, "{} {:?}", stringify!(#struct_name), (#(#get_selfs),*))
                    }
                }
            };
            implemented_show
            
            } else {
                panic!("sorry, may it's a complicated struct.");
            }
        }
        _ => panic!("sorry, Show is not implemented for union or enum type.")
    };
    expanded.into()
}


// attribute macro
#[proc_macro_attribute]
pub fn rust_decorator(attr: TokenStream, func: TokenStream) -> TokenStream {
    let func = parse_macro_input!(func as ItemFn); // 我们传入的是一个函数，所以要用到ItemFn
    let func_vis = &func.vis; // pub
    let func_block = &func.block; // 函数主体实现部分{}

    let func_decl = &func.sig; // 函数申明
    let func_name = &func_decl.ident; // 函数名
    let func_generics = &func_decl.generics; // 函数泛型
    let func_inputs = &func_decl.inputs; // 函数输入参数
    let func_output = &func_decl.output; // 函数返回

    // 提取参数，参数可能是多个
    let params: Vec<_> = func_inputs.iter().map(|i| {
        match i {
            // 提取形参的pattern
            // https://docs.rs/syn/1.0.1/syn/struct.PatType.html
            FnArg::Typed(ref val) => &val.pat, // pat没有办法移出val，只能借用，或者val.pat.clone()
            _ => unreachable!("it's not gonna happen."),
        }
    }).collect();
    
    // 解析attr
    let attr = parse_macro_input!(attr as AttributeArgs);
    // 提取attr的ident，此处例子只有一个attribute
    let attr_ident = match attr.get(0).as_ref().unwrap() {
        NestedMeta::Meta(Meta::Path(ref attr_ident)) => attr_ident.clone(),
        _ => unreachable!("it not gonna happen."),
    };
    
    // 创建新的ident, 例子里这个ident的名字是time_measure
    // let attr = Ident::new(&attr.to_string(), Span::call_site());
    let expanded = quote! { // 重新构建函数执行
        #func_vis fn #func_name #func_generics(#func_inputs) #func_output {
            // 这是没有重新构建的函数，最开始声明的，需要将其重建出来作为参数传入，
            // fn time_measure<F>(func: F) -> impl Fn(u64) where F: Fn(u64)
            // fn deco(t: u64) {
            //     let secs = Duration::from_secs(t);
            //     thread::sleep(secs);
            // }
            fn rebuild_func #func_generics(#func_inputs) #func_output #func_block
            // 注意这个#attr的函数签名：fn time_measure<F>(func: F) -> impl Fn(u64) where F: Fn(u64)
            // 形参是一个函数，就是rebuild_func
            let f = #attr_ident(rebuild_func);

            // 要修饰函数的参数，有可能是多个参数，所以这样匹配 #(#params,) *
            f(#(#params,) *)
        }
    };
    expanded.into()
}