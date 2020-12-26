#![feature(type_name_of_val)]
use core::any::type_name_of_val;

use std::str::FromStr;

fn main() {
    println!("输入多个数字，空格分隔");
    let mut line = String::new();
    let _len = std::io::stdin().read_line(&mut line).unwrap();
    // .map( |s| i32::from_str(s).unwrap())
    let mut parts = (*line).split(" ").map(|x| x.trim()).map(|x| i32::from_str(x).unwrap()).collect::<Vec<i32>>();
    println!("输入的是{0}", line);
    println!("type of parts: {}", type_name_of_val(&parts));
    println!("split to {:?}", parts);
    parts.sort();
    println!("sort result {:?}", parts);
}
