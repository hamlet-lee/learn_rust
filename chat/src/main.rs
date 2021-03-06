use std::collections::HashMap;
// use std::time::Duration;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::env;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time};
use std::sync::mpsc;
use console::Term;

extern crate log;
extern crate env_logger;

// learn tcpstream from https://www.jianshu.com/p/ec219de3132d

// learn debug log: https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/log.html
// and https://blog.csdn.net/s_lisheng/article/details/78250340
// run with: RUST_LOG=debug cargo run ...


// learn terminal from https://github.com/mitsuhiko/console
// or https://crates.io/crates/console
fn readline() -> String {
    let mut line: String = String::new();
    io::stdin().read_line(&mut line).unwrap();
    // remove ending \n
    line[0..line.len()-1].to_owned()
}

fn flush() {
    io::stdout().flush().unwrap();
}

enum Action {
    CONTINUE,
    STOP
}


fn read_stream_line<F:FnMut(&str) -> Action> (mut stream: TcpStream, mut line_processor: F) -> io::Result<usize> {
    let mut buf = [0; 512];
    let mut cur_line: Vec<u8> = Vec::new();
    'outer: loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read <= 0 {
            log::debug!("bytes_read = {}", bytes_read);
            break;
        }
        log::debug!("read len: {}", bytes_read);
        for pos in 0..bytes_read {
            let ch = buf[pos];
            log::debug!("ch={}", ch);
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
    log::debug!("read_stream_line end!");
    Ok(0)
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

fn message_sender (mut stream: TcpStream, rx: Receiver<Msg>) {
    'outer: loop {
        let msg = rx.recv().unwrap();
        match msg {
            Msg::TextMsg(s) => {
                log::debug!("stream.write(rx.recv) : {}", s);
                stream.write(s.as_bytes());
                stream.write("\n".as_bytes());
            },
            Msg::EndMsg => {
                break 'outer;
            }
        }

        // check end
    }
}

fn handle_client(stream: TcpStream, mut mp: MessagePusherV2) {
    let mut client_name: String = "UNKNOWN".to_owned();

    let (rx, tx) = mp.new_pair();
    
    let stream_for_sender = stream.try_clone().unwrap();
    thread::spawn( || {
        message_sender(stream_for_sender, rx);
    });

    read_stream_line(stream, move |s| {
        if s == "bye" {
            let msg = format!("{} 离开了 ...", client_name);
            println!("{}", msg);
            tx.send(Msg::TextMsg(msg));
            tx.send(Msg::EndMsg);
            Action::STOP
        } else if s.starts_with("name: ") {
            client_name = s["name: ".len()..].to_owned();
            let msg = format!("{} 进来了 ...", client_name);
            println!("{}", msg);
            tx.send(Msg::TextMsg(msg));
            Action::CONTINUE
        } else {
            let msg = format!("{} 说: {}", client_name, s);
            println!("{}", msg);
            tx.send(Msg::TextMsg(msg));
            Action::CONTINUE
        }
    }).unwrap_or(0);
}

enum Msg {
    TextMsg (String),
    EndMsg
}

struct MessagePusherInner {
    sender_map: HashMap<u64, Sender<Msg>>,
    next_id: u64
}

#[derive(Clone)]
struct MessagePusher {
    inner: Arc<Mutex<MessagePusherInner>>
}

#[derive(Clone)]
struct MessagePusherV2 {
    sender_map: Arc<Mutex<HashMap<u64, Sender<Msg>>>>,
    next_id: Arc<Mutex<u64>>
}

impl MessagePusher {
    fn new() -> MessagePusher {
        MessagePusher {
            inner: Arc::new(Mutex::new(MessagePusherInner {
                sender_map: HashMap::new(),
                next_id: 1
            }))
        }
    }

    // ret 1: for receive msg (sent by anyone)
    // ret 2: for send msg (to everyone)
    fn new_pair(&mut self) -> (Receiver<Msg>, Sender<Msg>) {
        let (tx_op, rx_peer) = mpsc::channel();

        let local_self = &self.inner;

        let cur_id;
        {
            let mut state = local_self.lock().unwrap();
            cur_id = state.next_id;
            state.next_id += 1;
            state.sender_map.insert(cur_id, tx_op.clone());
        }

        let (tx_peer, rx_op) = mpsc::channel();

        let self_in_thread = self.inner.clone();
        // 1) learn from https://users.rust-lang.org/t/how-to-use-self-while-spawning-a-thread-from-method/8282
        // 2) learn from https://www.philipdaniels.com/blog/2020/self-cloning-for-multiple-threads-in-rust/

        let other_self = self.clone();  // 2)
        thread::spawn( move | | {
            'outer: loop {
                let s = rx_op.recv().unwrap();
                match s {
                    Msg::TextMsg(txt) => {
                        // let mut inner = self_in_thread.lock().unwrap();
                        // MessagePusher::push_msg(&mut inner.sender_map, &txt);
                        other_self.push_msg_simple(&txt);
                    }
                    Msg::EndMsg => {
                        let mut inner = self_in_thread.lock().unwrap();
                        MessagePusher::close(&mut inner.sender_map, cur_id);
                        break 'outer;
                    }
                }
            }
        });

        (rx_peer, tx_peer)
    }

    fn push_msg_simple(&self, msg: &str) {
        log::debug!("push_msg : {}", msg);
        for (id, tx) in &self.inner.lock().unwrap().sender_map {
            log::debug!("tx({}).send : {}", id, msg);
            tx.send(Msg::TextMsg(msg.to_owned())).unwrap();
        }
    }

    fn push_msg(sender_map: &mut HashMap<u64, Sender<Msg>>, msg: &str) {
        log::debug!("push_msg : {}", msg);
        for (id, tx) in sender_map {
            log::debug!("tx({}).send : {}", id, msg);
            tx.send(Msg::TextMsg(msg.to_owned())).unwrap();
        }
    }

    fn close (sender_map: &mut HashMap<u64, Sender<Msg>>, id: u64) {
        let sender = sender_map.remove(&id);
        if let Some(s) = sender {
            s.send(Msg::EndMsg).unwrap();
        }
    }
}

impl MessagePusherV2 {
    fn new() -> MessagePusherV2 {
        MessagePusherV2 {
            sender_map: Arc::new(Mutex::new(
                HashMap::new()
            )),
            next_id: Arc::new(Mutex::new(1))
        }
    }

    // ret 1: for receive msg (sent by anyone)
    // ret 2: for send msg (to everyone)
    fn new_pair(&mut self) -> (Receiver<Msg>, Sender<Msg>) {
        let (tx_op, rx_peer) = mpsc::channel();

        let local_self = &self;

        let cur_id;
        {
            let mut locked_next_id = local_self.next_id.lock().unwrap();
            cur_id = *locked_next_id;
            *locked_next_id += 1;
        }

        {
            local_self.sender_map.lock().unwrap().insert(cur_id, tx_op.clone());
        }

        let (tx_peer, rx_op) = mpsc::channel();

        // 1) learn from https://users.rust-lang.org/t/how-to-use-self-while-spawning-a-thread-from-method/8282
        // 2) learn from https://www.philipdaniels.com/blog/2020/self-cloning-for-multiple-threads-in-rust/

        let other_self = self.clone();  // 2)
        thread::spawn( move | | {
            'outer: loop {
                let s = rx_op.recv().unwrap();
                match s {
                    Msg::TextMsg(txt) => {
                        other_self.push_msg(&txt);
                    }
                    Msg::EndMsg => {
                        other_self.close(cur_id);
                        break 'outer;
                    }
                }
            }
        });

        (rx_peer, tx_peer)
    }

    fn push_msg(&self, msg: &str) {
        log::debug!("push_msg : {}", msg);
        //* 解开MutextGuard，得到对象; & 采用借用方式
        for (id, tx) in &*(self.sender_map.lock().unwrap()) {
            log::debug!("tx({}).send : {}", id, msg);
            tx.send(Msg::TextMsg(msg.to_owned())).unwrap();
        }
    }

    fn close (&self, id: u64) {
        let sender = self.sender_map.lock().unwrap().remove(&id);
        if let Some(s) = sender {
            s.send(Msg::EndMsg).unwrap();
        }
    }
}

fn serve(port: u16, mp: MessagePusherV2) -> std::io::Result<()> {
    println!("Listening on port {} ...", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;

    for stream in listener.incoming() {
        println!("A client is connected!");
        let stream = stream.unwrap();
        let mp_cloned = mp.clone();
        std::thread::spawn(move || {
            handle_client(stream, mp_cloned);
        });

    }
    Ok(())
}

fn recv_print(stream: TcpStream, prompt: &str) {
    let term = Term::stdout();
    read_stream_line(stream, |s| {
        log::debug!("recv_print: {}", s);
        // println!("{}", s);
        term.clear_last_lines(1).unwrap();
        // thread::sleep(Duration::from_millis(1000));
        term.write_line(s).unwrap();
        // thread::sleep(Duration::from_millis(1000));
        term.clear_line().unwrap();
        // thread::sleep(Duration::from_millis(1000));
        term.write_str("\n");
        // thread::sleep(Duration::from_millis(1000));
        term.write_str(prompt);
        Action::CONTINUE
    }).unwrap_or(0);
}

fn client(host:&str, port: u16) {
    println!("connecting to {}:{} ... ", host, port);
    let addr = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(addr).unwrap();
    // 参考：https://stackoverflow.com/questions/28300742/how-do-i-share-a-socket-between-a-thread-and-a-function
    // 可以 clone stream !

    
    print!("请输入您的昵称（回车发送）: ");
    flush();
    let name = readline();
    stream.write(format!("name: {}\n", name).as_bytes()).unwrap();

    let stream_for_read = stream.try_clone().unwrap();
    let has_end = Arc::new(Mutex::new(false));
    let has_end2 = has_end.clone();
    std::thread::spawn(move || {
        recv_print(stream_for_read, "请输入信息(回车发送): ");
        log::debug!("setting has_end = true");
        let mut he = has_end2.lock().unwrap();
        *he = true;
        log::debug!("setted has_end ...");
    });

    let term = Term::stdout();
    'outer: loop {
        // 等待100毫秒，这样如果有信息，可以先显示
        thread::sleep(time::Duration::from_millis(100));

        // print!("your message: ");
        // flush();
        let input = readline();

        term.clear_last_lines(1).unwrap();

        stream.write(input.as_bytes()).unwrap();
        stream.write("\n".as_bytes()).unwrap();

        // 等待100毫秒，这样如果has end触发了，就结束。
        thread::sleep(time::Duration::from_millis(100));

        {
            let he = has_end.lock().unwrap();
            log::debug!("check has_end = {}", *he);
            if *he  {
                break 'outer;
            }
        }
    }
}

fn usage() {
    println!("usage 1: serve <port>");
    println!("usage 2: client <host> <port>");
}
fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        usage();
        return;
    }
    if args[1] == "serve" {
        let mp = MessagePusherV2::new();
        let port = args[2].parse::<u16>().unwrap();
        serve(port,  mp).unwrap();
    } else if args[1] == "client" {
        let host = &args[2];
        let port = args[3].parse::<u16>().unwrap();
        client(host, port);
    } else {
        usage();
    }
}
