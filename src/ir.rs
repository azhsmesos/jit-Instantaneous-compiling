use crate::brainfuck_open_code::OpenCode;
use std::io::{Read, Write};

mod brainfuck_open_code;

#[derive(Debug, PartialEq)]
pub enum IR {
    SHR(u32),
    SHL(u32),
    ADD(u8),
    SUB(u8),
    PUTCHAR,
    GETCHAR,
    JIZ(u32), // jump if zero
    JNZ(u32), // jump if not zero
}

pub struct BrainfuckCodeIR {
    pub instrs: Vec<IR>,
}

impl BrainfuckCodeIR {
    pub fn from(data: Vec<OpenCode>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut instrs: Vec<IR> = Vec::new();
        let mut jstack: Vec<u32> = Vec::new();

        // 判断是否是连续相同的指令
        for e in data {
            match e {
                OpenCode::SHR => match instrs.last_mut() {
                    Some(IR::SHR(x)) => {
                        *x += 1;
                    }
                    _ => {
                        instrs.push(IR::SHR(1));
                    }
                },
                OpenCode::SHL => match instrs.last_mut() {
                    Some(IR::SHL(x)) => {
                        *x += 1;
                    }
                    _ => {
                        instrs.push(IR::SHL(1));
                    }
                },

                OpenCode::ADD => match instrs.last_mut() {
                    Some(IR::ADD(x)) => {
                        let (b, _) = x.overflowing_add(1);
                        *x = b;
                    }
                    _ => {
                        instrs.push(IR::ADD(1));
                    }
                },
                OpenCode::SUB => match instrs.last_mut() {
                    Some(IR::SUB(x)) => {
                        let (b, _) = x.overflowing_sub(1);
                        *x = b;
                    }
                    _ => {
                        instrs.push(IR::SUB(1));
                    }
                },
                OpenCode::PUTCHAR => {
                    instrs.push(IR::PUTCHAR);
                }
                OpenCode::GETCHAR => {
                    instrs.push(IR::GETCHAR);
                }
                OpenCode::LB => {
                    instrs.push(IR::JIZ(0));
                    jstack.push((instrs.len() - 1) as u32);
                }
                OpenCode::RB => {
                    let js = jstack.pop().ok_or("pop from empty list")?;
                    instrs.push(IR::JNZ(js));
                    let instrs_len = instrs.len();
                    match &mut instrs[js as usize] {
                        IR::JIZ(x) => {
                            *x = (instrs_len - 1) as u32;
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }

        Ok(BrainfuckCodeIR { instrs })
    }
}

struct Interpreter {
    stack: Vec<u8>,
}

impl Interpreter {
    fn run(&mut self, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let opcode_code = brainfuck_open_code::BrainfuckCode::from(data)?;
        let ir_code = BrainfuckCodeIR::from(opcode_code.instrs)?;
        let ir_code_len = ir_code.instrs.len();
        let mut brainfuck_pc = 0;
        let mut brainfuck_point = 0;

        loop {
            if brainfuck_pc >= ir_code_len {
                break;
            }
            match ir_code.instrs[brainfuck_pc] {
                IR::SHR(x) => {
                    brainfuck_point += x as usize;
                    if brainfuck_point >= self.stack.len() {
                        let expand = brainfuck_point - self.stack.len() + 1;
                        for _ in 0..expand {
                            self.stack.push(0);
                        }
                    }
                }
                IR::SHL(x) => loop {
                    for i in 0..x {
                        if brainfuck_point != 0 {
                            brainfuck_point -= 1;
                        } else {
                            break;
                        }
                    }
                },
                IR::ADD(x) => {
                    // 防止溢出
                    self.stack[brainfuck_point] = self.stack[brainfuck_point].overflowing_add(x).0;
                }
                IR::SUB(x) => {
                    self.stack[brainfuck_point] = self.stack[brainfuck_point].overflowing_sub(1).0;
                }
                IR::PUTCHAR => {
                    std::io::stdout().write_all(&[self.stack[brainfuck_point]])?;
                }
                IR::GETCHAR => {
                    let mut buf: Vec<u8> = vec![0; 1];
                    std::io::stdin().read_exact(&mut buf)?;
                    self.stack[brainfuck_point] = buf[0];
                }
                IR::JIZ(x) => {
                    if self.stack[brainfuck_point] == 0x00 {
                        brainfuck_pc = x as usize;
                    }
                }
                IR::JNZ(x) => {
                    if self.stack[brainfuck_point] != 0x00 {
                        brainfuck_pc = x as usize;
                    }
                }
            }
            brainfuck_pc += 1;
        }

        Ok(())
    }
}

impl std::default::Default for Interpreter {
    fn default() -> Self {
        // 初始化，提供默认值
        Self { stack: vec![0; 1] }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1])?;

    // let code = brainfuck_opcode::OpenCode::from(data)?;
    // let ir_code = BrainfuckCode::from(code.instrs)?;

    // println!("istrs; {:?}", ir_code.instrs);

    let mut interpreter = Interpreter::default();
    interpreter.run(data);

    Ok(())
}
