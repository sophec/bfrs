use std::env;
use std::fmt::{Display, Error, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;

enum Op {
    Move(i32),
    Add(i32),
    MoveAdd(i32, i32),
    Print,
    Read,
    LoopStart,
    LoopEnd,
}

impl Op {
    fn try_parse(c: u8) -> Option<Op> {
        match c {
            b'>' => Some(Op::Move(1)),
            b'<' => Some(Op::Move(-1)),
            b'+' => Some(Op::Add(1)),
            b'-' => Some(Op::Add(-1)),
            b'.' => Some(Op::Print),
            b',' => Some(Op::Read),
            b'[' => Some(Op::LoopStart),
            b']' => Some(Op::LoopEnd),
            _ => None,
        }
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Op::MoveAdd(mv @ (1 | -1), add @ (1 | -1)) => write!(f, "{}*{}p;",
                if *add < 0 { "--" } else { "++" },
                if *mv < 0 { "--" } else { "++" }
            ),
            Op::MoveAdd(mv @ (1 | -1), add) => write!(f, "*{}p{}={};",
                if *mv < 0 { "--" } else { "++" },
                if *add < 0 { '-' } else { '+' },
                add.abs()
            ),
            Op::MoveAdd(mv, add @ (1 | -1)) => write!(f, "{}*(p{}={});",
                if *add < 0 { "--" } else { "++" },
                if *mv < 0 { '-' } else { '+' },
                mv.abs()
            ),
            Op::MoveAdd(mv, add) => write!(f, "*(p{}={}){}={};",
                if *mv < 0 { '-' } else { '+' },
                mv.abs(),
                if *add < 0 { '-' } else { '+' },
                add.abs()
            ),
            Op::Move(-1) => write!(f, "--p;"),
            Op::Move(1) => write!(f, "++p;"),
            Op::Move(amt) => write!(f, "p{}={};", if *amt < 0 { '-' } else { '+' }, amt.abs()),
            Op::Add(-1) => write!(f, "--*p;"),
            Op::Add(1) => write!(f, "++*p;"),
            Op::Add(amt) => write!(f, "*p{}={};", if *amt < 0 { '-' } else { '+' }, amt.abs()),
            Op::Print => write!(f, "putchar(*p);"),
            Op::Read => write!(f, "*p=getchar();"),
            Op::LoopStart => write!(f, "for(;*p;){{"),
            Op::LoopEnd => write!(f, "}}"),
        }
    }
}

struct Program {
    ops: Vec<Op>,
}

impl Program {
    fn new() -> Self {
        Program { ops: Vec::new() }
    }

    fn push_op(&mut self, op: Op) {
        if self.ops.len() == 0 {
            self.ops.push(op);
            return;
        }

        let last_op = self.ops.last_mut().unwrap();

        match op {
            Op::Move(amt) => match last_op {
                Op::Move(last_amt) => {
                    *last_amt += amt;
                    if *last_amt == 0 {
                        self.ops.pop();
                    }
                },
                _ => self.ops.push(op),
            },
            Op::Add(amt) => match last_op {
                Op::Add(last_amt) => {
                    *last_amt += amt;
                    if *last_amt == 0 {
                        self.ops.pop();
                    }
                },
                Op::Move(mv) => {
                    *last_op = Op::MoveAdd(*mv, amt);
                },
                Op::MoveAdd(mv, last_amt) => {
                    *last_amt += amt;
                    if *last_amt == 0 {
                        *last_op = Op::Move(*mv);
                    }
                },
                _ => self.ops.push(op),
            },
            Op::Print | Op::Read | Op::LoopStart | Op::MoveAdd(_, _) => self.ops.push(op),
            Op::LoopEnd=> match last_op {
                Op::LoopStart => { self.ops.pop(); () },
                _ => self.ops.push(op),
            },
        }
    }

    fn finalize(&mut self) {
        // Remove loop at the start, if any. This is just dead code.
        if let Op::LoopStart = self.ops.first().unwrap() {
            let mut level = 0i32;
            let mut end_idx: usize = 0;

            for (index, op) in self.ops.iter().enumerate() {
                match *op {
                    Op::LoopStart => level += 1,
                    Op::LoopEnd => level -= 1,
                    _ => (),
                }
                if level == 0 {
                    end_idx = index;
                    break;
                }
            }

            // if not, this is an invalid program and we will let the C compiler handle that for
            // now since I am too lazy to return an error :)
            if level == 0 {
                self.ops.drain(..end_idx + 1);
            }
        }
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#include <stdio.h>\nstatic char a[65535]={{0}};static char*p=a;int main(){{")?;

        for op in &self.ops {
            write!(f, "{op}")?;
        }

        write!(f, "}}")
    }
}

fn fuck<R>(reader: &mut BufReader<R>) -> Result<Program, &'static str> where R: std::io::Read {
    let mut bytes = reader.bytes();
    let mut program = Program::new();

    while let Some(r) = bytes.next() {
        match r.map(Op::try_parse) {
            Ok(Some(op)) => program.push_op(op),
            Err(_) => return Err("file IO error"),
            _ => (),
        }
    }

    program.finalize();

    Ok(program)
}

fn usage(argv0: &str) {
    eprintln!(r"usage: {} [FILE]
  If a FILE is missing or '-', reads from stdin.", argv0);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        usage(&args[0]);
        exit(1);
    }

    let res = if args.len() == 2 && args[1] != "-" {
        let path = Path::new(&args[1]);
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("error opening file: {e}");
                exit(1);
            },
        };

        let mut buf_reader = BufReader::new(file);
        fuck(&mut buf_reader)
    } else {
        let mut buf_reader = BufReader::new(std::io::stdin());
        fuck(&mut buf_reader)
    };

    match res {
        Ok(program) => println!("{program}"),
        Err(e) => eprintln!("{e}"),
    }
}
