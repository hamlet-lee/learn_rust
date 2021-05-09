// https://crates.io/crates/sqlite
// if use nom:
//   https://zhuanlan.zhihu.com/p/115017849
//   https://crates.io/crates/nom

// https://crates.io/crates/regex
// https://docs.rs/regex/1.5.4/regex/#syntax

// file
// https://www.cnblogs.com/dhcn/p/12947090.html
use std::io;
use std::io::Write;
use crate::Cmd::{QuitCmd, Unknown, LoadCmd, SelectCmd};
use regex::Regex;
use std::fs::File;
use std::io::Read;

enum Cmd {
    // "quit"
    QuitCmd,
    // "load /path/to/file as tbl1"
    LoadCmd { file: String, tbl: String},
    SelectCmd { sql: String },
    Unknown { cmd: String }
}

fn parseCmd (input: &str) -> Cmd {
    let re = Regex::new(r"load\s+(?P<file>\S+)\s+as\s+(?P<tbl>\S+)").unwrap();
    // let re = Regex::new(r"load\s+(?P<file>\S+)\s+as").unwrap();
    if input == "quit" {
        QuitCmd
    } else {
        let capsRes = re.captures(input);
        if let Some(caps) = capsRes {
            LoadCmd { file: caps["file"].to_owned(), tbl: caps["tbl"].to_owned()}
        } else {
            if input.starts_with("select") {
                SelectCmd { sql: input.to_owned() }
            } else {
                Unknown { cmd: input.to_owned() }
            }
        }
    }
}

fn main() {
    let connection = sqlite::open(":memory:").unwrap();
    let mut line = String::new();
    let stdout = io::stdout();
    let stdin = io::stdin();
    loop {
        print!(">");
        io::stdout().flush().unwrap();
        line.clear();
        stdin.read_line(&mut line).unwrap();
        let cmdStr = line.trim();
        let cmd = parseCmd(cmdStr);

        match cmd {
            QuitCmd => {
                println!("Bye!");
                break;
            },
            LoadCmd {file, tbl} => {
                println!("load file: {} into tbl: {}", file, tbl);
                let mut f = File::open(file).unwrap();
                let mut s: String = String::new();
                f.read_to_string(&mut s);
                let lines = s.split("\n");
                let mut no = 0;
                // header
                let mut headerVec = Vec::new();
                // val
                let mut valVec = Vec::new();
                for line in lines {
                    println!("line {}: {} ", no, line);
                    if line.len() > 0 {
                        if no == 0 {
                            // head line
                            println!("parsing head line ...");
                            for colName in line.split("\t") {
                                headerVec.push(colName.to_owned());
                            }
                            for hdr in &headerVec {
                                println!(" header found: [{}]", hdr);
                            }
                        } else {
                            // data line
                            println!("parsing val line {} ...", no);
                            let mut lineVal = Vec::new();
                            for colVal in line.split("\t") {
                                lineVal.push(colVal.to_owned());
                            }
                            valVec.push(lineVal);
                        }
                    }
                    no += 1;
                }

                let mut createStmt: String = "CREATE TABLE ".into();
                createStmt.push_str(&tbl);
                createStmt.push_str(" (");

                let mut colNo = 0;
                for hdr in headerVec {
                    if colNo > 0 {
                        createStmt.push_str(", ");
                    }
                    createStmt.push_str( &format!("{}  INTEGER", hdr));
                    colNo += 1;
                }
                createStmt.push_str(");");

                println!("executing: {}", createStmt);
                connection.execute(createStmt).unwrap();

                for valLine in valVec {
                    let valsStr = valLine.join(",");
                    let insertStmt = format!("INSERT INTO {} VALUES ({});", tbl, valsStr);
                    println!("executing: {}", insertStmt);
                    connection.execute(insertStmt).unwrap();
                }

            },
            SelectCmd {sql} => {

            },
            _ => {
                println!("received [{}]", cmdStr);
            }
        }
    }
}
