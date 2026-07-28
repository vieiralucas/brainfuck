use std::{io::Read, process::exit};

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
    type Target = Vec<Token>;

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
        if self.ip >= self.program.len() - 1 {
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
                    eprintln!("stack underflow");
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
                print!("{}", self.data[self.ptr] as char);
            }
            TokenKind::Input => {
                let mut buf = [0u8; 1];
                let read = std::io::stdin().read(&mut buf).expect("read stdin");
                if read != 0 {
                    self.data[self.ptr] = buf[0];
                }
            }
            TokenKind::LoopStart => {
                if self.data[self.ptr] == 0 {
                    let mut balance = 1;
                    while balance != 0 {
                        self.ip += 1;
                        if self.program.len() <= self.ip {
                            eprintln!("syntax error, matching bracket not found");
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
                } else {
                    eprintln!("syntax error, matching bracket not found");
                    exit(1);
                }
                return true;
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

fn parse(program: &str) -> Program {
    let tokens = program
        .lines()
        .enumerate()
        .flat_map(|(line, line_str)| {
            line_str.chars().enumerate().filter_map(move |(col, c)| {
                let kind = match c {
                    '>' => Some(TokenKind::IncPtr),
                    '<' => Some(TokenKind::DecPtr),
                    '+' => Some(TokenKind::IncVal),
                    '-' => Some(TokenKind::DecVal),
                    '.' => Some(TokenKind::Output),
                    ',' => Some(TokenKind::Input),
                    '[' => Some(TokenKind::LoopStart),
                    ']' => Some(TokenKind::LoopEnd),
                    _ => None,
                };

                kind.map(|kind| Token {
                    kind,
                    line: line + 1,
                    col: col + 1,
                })
            })
        })
        .collect();

    Program(tokens)
}

fn main() {
    let program_path = std::env::args().nth(1).expect("no program path provided");
    let program = std::fs::read_to_string(program_path).expect("could not read program file");
    let program = parse(&program);
    program.validate();

    let mut machine = Machine::new(program);
    while machine.step() {}
}
