use proc_macro_example::{ my_proc_macro, Show, rust_decorator };
// use crate::{ Show, builtin_decorator }; // crate doesn't work，因为目前过程宏只能作为单独的crate存在

use std::time::{self, Duration, SystemTime};
use std::thread;


// 过程宏不能作为语句放在main里面，只能放在main外面。
// 这样我们生成了一个名为test_proc的函数，可以在main里面调用了。
my_proc_macro!(proc);
// 生成了一个名为test_macro的函数。
my_proc_macro!(macro);


// 使用了Show，就不需要去实现Display trait了，或者#[derive(Debug)],
// 这样实现了代码的复用。
#[derive(Show)]
struct T{
    i: i32,
    u: u32,
    s: String,
    t: SystemTime,
}


// 该函数接受一个函数作为参数，并返回一个闭包，代码很简单，就不解释了。
fn runtime_measurement<F>(func: F) -> impl Fn(u64) where F: Fn(u64) {
    move |s| {
        let start = time::Instant::now();
        func(s);
        println!("time cost {:?}", start.elapsed());
    }
}


// 这就是属性过程宏的使用方式，#[my_proc_macro_attribute(attr)]，
// 可以对应到这个过程宏的定义
#[rust_decorator(runtime_measurement)]
fn deco(t: u64) {
    let secs = Duration::from_secs(t);
    thread::sleep(secs);
}


fn main() {
    println!("proc macro: ");
    test_proc(2);
    test_macro("proc-macro func");

    println!("proc macro derive: ");
    let t1 = T{i: 323, u: 12, s: "proc-macro".to_string(), t: SystemTime::now()};
    println!("{}", t1);

    println!("proc macro attribute: ");
    deco(4);
    deco(2);
}