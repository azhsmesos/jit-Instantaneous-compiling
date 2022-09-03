mod brainfuck_open_code;

use brainfuck_open_code::{BrainfuckCode, OpenCode};
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1])?;
    // let code = Code::from(data)?;
    let mut interpreter = Interpreter::new();
    interpreter.run(data);

    // println!("{:?}", code.instrs);

    Ok(())
}

// brainfuck 解释器
struct Interpreter {
    // 象征一条输出的无限长的纸带
    stack: Vec<u8>,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            // default 初始化
            stack: vec![0; 1],
        }
    }

    fn run(&mut self, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let code = BrainfuckCode::from(data)?;
        let code_len = code.instrs.len();
        let mut brainfuck_pc: usize = 0; //  程序计数器，代码执行到哪儿了，执行到哪个指令
        let mut brainfuck_point: usize = 0; //  纸带上面的指针，索引在哪儿

        // interpreter主循环
        loop {
            if brainfuck_pc >= code_len {
                break;
            }
            match code.instrs[brainfuck_pc] {
                OpenCode::SHR => {
                    brainfuck_point += 1;
                    if brainfuck_point == self.stack.len() {
                        self.stack.push(0);
                    }
                }
                OpenCode::SHL => {
                    if brainfuck_point != 0 {
                        brainfuck_point -= 1;
                    }
                }
                OpenCode::ADD => {
                    // 防止溢出
                    self.stack[brainfuck_point] = self.stack[brainfuck_point].overflowing_add(1).0;
                }
                OpenCode::SUB => {
                    self.stack[brainfuck_point] = self.stack[brainfuck_point].overflowing_sub(1).0;
                }
                OpenCode::PUTCHAR => {
                    std::io::stdout().write_all(&[self.stack[brainfuck_point]])?;
                }
                OpenCode::GETCHAR => {
                    let mut buf: Vec<u8> = vec![0; 1];
                    std::io::stdin().read_exact(&mut buf)?;
                    self.stack[brainfuck_point] = buf[0];
                }
                OpenCode::LB => {
                    if self.stack[brainfuck_point] == 0x00 {
                        brainfuck_pc = code.jtable[&brainfuck_pc];
                    }
                }
                OpenCode::RB => {
                    if self.stack[brainfuck_point] != 0x00 {
                        brainfuck_pc = code.jtable[&brainfuck_pc];
                    }
                }
            }
            brainfuck_pc += 1;
        }

        Ok(())
    }
}
