use std::{
    io::{BufReader, Read, Write, stdout},
    process::exit,
};

#[derive(PartialEq, Clone, Copy, Debug)]
enum TokenKind {
    IncPtr,
    DecPtr,
    IncVal,
    DecVal,
    Output,
    Input,
    LoopStart,
    LoopEnd,
}

#[derive(PartialEq, Clone, Copy, Debug)]
struct Token {
    kind: TokenKind,
    line: usize,
    col: usize,
}

struct Program(Vec<Token>);

impl std::ops::Deref for Program {
    type Target = [Token];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Program {
    fn validate(&self) {
        let mut open_brackets = Vec::new();
        for c in self.iter() {
            match c.kind {
                TokenKind::LoopStart => {
                    open_brackets.push(c);
                }
                TokenKind::LoopEnd if open_brackets.pop().is_none() => {
                    eprintln!(
                        "{}:{} syntax error, unmatched closing bracket",
                        c.line, c.col
                    );
                    exit(1);
                }
                _ => {}
            }
        }

        if let Some(last_open) = open_brackets.last() {
            eprintln!(
                "{}:{} syntax error, unmatched opening bracket",
                last_open.line, last_open.col
            );
            exit(1);
        }
    }
}

struct Machine {
    program: Program,
    data: Vec<u8>,
    ptr: usize,
    ip: usize,
    loop_stack: Vec<usize>,
}

impl Machine {
    fn new(program: Program) -> Self {
        Self {
            program,
            data: vec![0; 0],
            ptr: 0,
            ip: 0,
            loop_stack: Vec::new(),
        }
    }

    fn step(&mut self) -> bool {
        if self.ip >= self.program.len() {
            return false;
        }

        if self.data.len() <= self.ptr {
            self.data.resize(self.ptr + 1, 0);
        }

        let c = &self.program[self.ip];

        match c.kind {
            TokenKind::IncPtr => self.ptr += 1,
            TokenKind::DecPtr => {
                if self.ptr == 0 {
                    eprintln!("{}:{} tape underflow", c.line, c.col);
                    exit(1);
                }
                self.ptr -= 1;
            }
            TokenKind::IncVal => {
                if self.data[self.ptr] == 255 {
                    self.data[self.ptr] = 0;
                } else {
                    self.data[self.ptr] += 1;
                }
            }
            TokenKind::DecVal => {
                if self.data[self.ptr] == 0 {
                    self.data[self.ptr] = 255;
                } else {
                    self.data[self.ptr] -= 1;
                }
            }
            TokenKind::Output => {
                let mut out = stdout();
                let r = out
                    .write_all(&[self.data[self.ptr]])
                    .and_then(|_| out.flush());
                if let Err(e) = r {
                    if e.kind() == std::io::ErrorKind::BrokenPipe {
                        exit(0);
                    }
                    eprintln!("{}:{} error writing to stdout: {}", c.line, c.col, e);
                    exit(1);
                }
            }
            TokenKind::Input => {
                let mut buf = [0u8; 1];
                let n = loop {
                    match std::io::stdin().read(&mut buf) {
                        Ok(n) => break n,
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(e) => {
                            eprintln!("{}:{} error reading stdin: {}", c.line, c.col, e);
                            exit(1);
                        }
                    }
                };

                if n == 0 {
                    self.data[self.ptr] = 0;
                } else {
                    self.data[self.ptr] = buf[0];
                }
            }
            TokenKind::LoopStart => {
                if self.data[self.ptr] == 0 {
                    let mut balance = 1;
                    while balance != 0 {
                        self.ip += 1;
                        if self.program.len() <= self.ip {
                            eprintln!(
                                "{}:{} syntax error, matching bracket not found",
                                c.line, c.col
                            );
                            exit(1);
                        }

                        if self.program[self.ip].kind == TokenKind::LoopStart {
                            balance += 1;
                        } else if self.program[self.ip].kind == TokenKind::LoopEnd {
                            balance -= 1;
                        }
                    }
                    self.ip += 1;
                    return true;
                } else {
                    self.loop_stack.push(self.ip);
                }
            }
            TokenKind::LoopEnd => {
                if let Some(loop_start) = self.loop_stack.pop() {
                    self.ip = loop_start;
                    return true;
                }

                eprintln!(
                    "{}:{} syntax error, matching bracket not found",
                    c.line, c.col
                );
                exit(1);
            }
        }
        self.ip += 1;
        true
    }

    #[allow(dead_code)]
    fn debug(&self) {
        println!(
            "\nip: {}, ptr: {}, data: {:?}\n",
            self.ip, self.ptr, self.data
        );
    }
}

fn parse<R: Read>(program: R) -> Program {
    let buf = BufReader::new(program);
    let (mut row, mut col) = (1, 1);
    let mut tokens = Vec::new();
    for byte_result in buf.bytes() {
        let b = match byte_result {
            Ok(b) => b,
            Err(e) => {
                eprintln!("error reading program file: {}", e);
                exit(1);
            }
        };

        let kind = match b {
            b'>' => Some(TokenKind::IncPtr),
            b'<' => Some(TokenKind::DecPtr),
            b'+' => Some(TokenKind::IncVal),
            b'-' => Some(TokenKind::DecVal),
            b'.' => Some(TokenKind::Output),
            b',' => Some(TokenKind::Input),
            b'[' => Some(TokenKind::LoopStart),
            b']' => Some(TokenKind::LoopEnd),
            _ => None,
        };

        if let Some(kind) = kind {
            tokens.push(Token {
                kind,
                line: row,
                col,
            });
        }

        match b {
            b'\n' => {
                row += 1;
                col = 1;
            }
            _ => {
                col += 1;
            }
        };
    }

    Program(tokens)
}

fn main() {
    let program_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("no program path provided");
        exit(1);
    });
    let file = std::fs::File::open(&program_path).unwrap_or_else(|e| {
        eprintln!("error opening program file {}: {}", program_path, e);
        exit(1);
    });
    let meta = file.metadata().unwrap_or_else(|e| {
        eprintln!("error reading program file {}: {}", program_path, e);
        exit(1);
    });
    if meta.is_dir() {
        eprintln!("{} is a directory", program_path);
        exit(1);
    }

    let program = parse(file);
    program.validate();

    let mut machine = Machine::new(program);
    while machine.step() {}
}
