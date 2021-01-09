use std::env;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::net::Shutdown;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool};

// learn from https://www.jianshu.com/p/ec219de3132d

fn readline() -> String {
    let mut line: String = String::new();
    io::stdin().read_line(&mut line);
    // remove ending \n
    line[0..line.len()-1].to_owned()
}

fn flush() {
    io::stdout().flush();
}

enum Action {
    CONTINUE,
    STOP
}


fn read_stream_line<F:FnMut(&str) -> Action> (mut stream: TcpStream, mut line_processor: F) {
    let mut buf = [0; 512];
    let mut cur_line: Vec<u8> = Vec::new();
    'outer: loop {
        let bytes_read = stream.read(&mut buf).unwrap();
        if bytes_read <= 0 {
            break;
        }
        println!("read len: {}", bytes_read);
        for pos in 0..bytes_read {
            let ch = buf[pos];
            println!("ch={}", ch);
            cur_line.push(ch);
            if ch == '\n' as u8 {
                let mut content_len = cur_line.len() - 1;
                if cur_line.len() > 1 && cur_line[cur_line.len() - 2] == '\r' as u8 {
                    content_len -= 1;
                }

                let s: String = String::from_utf8(cur_line[0..content_len].to_vec()).unwrap();
                
                match line_processor(s.as_str()) {
                    Action::STOP => {
                        break 'outer
                    },
                    _ => {}
                }
                cur_line.clear();
            }
        }
    }
    println!("read_stream_line end!");
}

// fn handle_client(mut stream: TcpStream) {
//     let mut buf = [0; 512];
//     let mut cur_line: Vec<u8> = Vec::new();
//     let mut end_of_chat = false;
//     let mut client_name: String = "UNKNOWN".to_owned();
//     loop {
//         let bytes_read = stream.read(&mut buf).unwrap();
//         if bytes_read <= 0 {
//             break;
//         }
//         println!("read len: {}", bytes_read);
//         for pos in 0..bytes_read {
//             let ch = buf[pos];
//             println!("ch={}", ch);
//             cur_line.push(ch);
//             if ch == '\n' as u8 {
//                 let mut content_len = cur_line.len() - 1;
//                 if cur_line.len() > 1 && cur_line[cur_line.len() - 2] == '\r' as u8 {
//                     content_len -= 1;
//                 }

//                 let s: String = String::from_utf8(cur_line[0..content_len].to_vec()).unwrap();
                
//                 if s == "bye" {
//                     end_of_chat = true;
//                     println!("{} 离开了 ...", client_name);
//                     break;
//                 } else if s.starts_with("name: ") {
//                     client_name = s["name: ".len()..].to_owned();
//                     println!("{} 进来了 ...", client_name);
//                 } else {
//                     println!("{} said: {}", client_name, s);
//                 }
//                 cur_line.clear();
//             }
//         }
//         if end_of_chat {
//             break;
//         }
//     }
// }

fn handle_client(stream: TcpStream) {
    let mut end_of_chat = false;
    let mut client_name: String = "UNKNOWN".to_owned();
    
    read_stream_line(stream, move |s| {
        if s == "bye" {
            end_of_chat = true;
            println!("{} 离开了 ...", client_name);
            Action::STOP
        } else if s.starts_with("name: ") {
            client_name = s["name: ".len()..].to_owned();
            println!("{} 进来了 ...", client_name);
            Action::CONTINUE
        } else {
            println!("{} 说: {}", client_name, s);
            Action::CONTINUE
        }
    })
    
}

fn serve(port: u16) -> std::io::Result<()> {
    println!("Listening on port {} ...", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    for stream in listener.incoming() {
        println!("A client is connected!");
        let stream = stream.unwrap();
        std::thread::spawn(move || {
            handle_client(stream);
        });
    }
    Ok(())
}

fn recv_print(mut stream: TcpStream) {
    read_stream_line(stream, |s| {
        println!("{}", s);
        Action::CONTINUE
    })
}

fn client(host:&str, port: u16) {
    println!("connecting to {}:{} ... ", host, port);
    let addr = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(addr).unwrap();
    // 参考：https://stackoverflow.com/questions/28300742/how-do-i-share-a-socket-between-a-thread-and-a-function
    // 可以 clone stream !

    let mut stream_for_read = stream.try_clone().unwrap();
    let mut has_end = Arc::new(Mutex::new(false));
    let mut has_end2 = has_end.clone();
    std::thread::spawn(move || {
        recv_print(stream_for_read);
        println!("setting has_end = true");
        let mut he = has_end2.lock().unwrap();
        *he = true;
        println!("setted has_end ...");
    });

    print!("your name: ");
    flush();
    let name = readline();
    stream.write(format!("name: {}\n", name).as_bytes());
    'outer: loop {
        {
            let he = has_end.lock().unwrap();
            println!("check has_end = {}", *he);
            if *he  {
                break 'outer;
            }
        }
        print!("your message: ");
        flush();
        let input = readline();
        stream.write(input.as_bytes());
        stream.write("\n".as_bytes());
    }
}

fn usage() {
    println!("usage 1: serve <port>");
    println!("usage 2: client <host> <port>");
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        usage();
        return;
    }
    if args[1] == "serve" {
        let port = args[2].parse::<u16>().unwrap();
        serve(port);
    } else if args[1] == "client" {
        let host = &args[2];
        let port = args[3].parse::<u16>().unwrap();
        client(host, port);
    } else {
        usage();
    }
}
