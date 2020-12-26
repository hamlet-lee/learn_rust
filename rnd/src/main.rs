#![feature(non_ascii_idents)]

use std::str::FromStr;
use std::{thread, time};
use std::io;
use std::io::Write;
extern crate rand;
use rand::Rng;

fn flush() {
    io::stdout().flush();
}

fn readline() -> String {
    let mut line:String = String::new();
    io::stdin().read_line(&mut line);
    line
}

fn main() {
    print!("您有几种选择？");
    flush();
    let line = readline();
    let 备选数 = i32::from_str(line.trim()).unwrap();

    let mut exp:Vec<String> = Vec::new();
    for i in 0..备选数 {
       print!("{} 代表：", i+1);
       flush();
       let line = readline();
       let e = line.trim().to_string();
    //    println!("note {}", e);
       exp.push(e);
    }

    print!("准备好了吗？准备好了按回车...");
    flush();
    readline();

    // println!("{} 选 1 ...", 备选数);
    for _i in 0..2 {
        // https://blog.csdn.net/hustlei/article/details/102511654
        print!("选,");
        flush();
        thread::sleep(time::Duration::from_secs(1));
    }
    for _i in 0..3 {
        // https://blog.csdn.net/hustlei/article/details/102511654
        print!("选!");
        flush();
        thread::sleep(time::Duration::from_millis(500));
    }
    println!("");

    let mut rng = rand::thread_rng();
    let c = rng.gen_range(0, 备选数);
    let e = &(exp[c as usize]);
    println!("给你选择了 {} => {}。惊不惊喜！意不意外！",
         c + 1, e);
    print!("按回车结束...");
    flush();
    readline();
}
