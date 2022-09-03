use crate::ir::IR;
use dynasm::dynasm;
use dynasmrt::{DynasmApi, DynasmLabelApi};
use std::io::prelude::*;
use std::io::Write;

mod brainfuck_open_code;
mod ir;

unsafe extern "sysv64" fn putchar(char: u8) {
    std::io::stdout().write_all(&[char]).unwrap()
}

#[derive(Debug, PartialEq, Default)]
struct Interpreter {}

impl Interpreter {
    fn run(&mut self, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        // 将brainfuck文件转换为OpenCode数组
        let brainfuck_code = brainfuck_open_code::BrainfuckCode::from(data)?;
        // 将OpenCode 转换为中间代码
        let ir_code = ir::BrainfuckCodeIR::from(brainfuck_code.instrs)?;

        // 匹配[] 时用来跳转
        let mut lr_loop = vec![];

        // 所有的可执行对象都写入该内存
        let mut exec_buffer = dynasmrt::x64::Assembler::new()?;
        let entry_point = exec_buffer.offset();

        dynasm! (exec_buffer
            ; .arch x64
            ; mov rcx, rdi
        );

        for ir in ir_code.instrs {
            match ir {
                IR::SHL(x) => dynasm! (exec_buffer
                    ; sub rcx, x as i32 //brainfuck_point -= x
                ),
                IR::SHR(x) => dynasm! (exec_buffer
                    ; add rcx, x as i32  // brainfuck_point += x
                ),
                IR::ADD(x) => dynasm! (exec_buffer
                    ; add BYTE [rcx], x as i8  // brainfuck_point* += x
                ),
                IR::SUB(x) => dynasm! (exec_buffer
                    ; sub BYTE [rcx], x as i8  // brainfuck_point* -= x
                ),
                // 输入
                IR::PUTCHAR => dynasm! (exec_buffer
                    ; mov r15, rcx
                    ; mov rdi, rcx
                    ; mov rax, QWORD putchar as _
                    ; sub rsp, BYTE 0x28
                    ; call rax
                    ; add rsp, BYTE 0x28
                    ; mov rcx, r15
                ),
                IR::GETCHAR => {}
                // IR::GETCHAR => dynasm! (exec_buffer
                //     ; mov r15, rcx
                //     ; mov rdi, rcx
                //     ; mov rax, QWORD getchar as _
                //     ; sub rsp, BYTE 0x28
                //     ; call rax
                //     ; add rsp, BYTE 0x28
                //     ; mov rcx, r15
                // ),
                IR::JIZ(_) => {
                    let l = exec_buffer.new_dynamic_label();
                    let r = exec_buffer.new_dynamic_label();
                    lr_loop.push((l, r));
                    // 如果指向的单元值为0，就向后跳转到对应的]指令处
                    dynasm! (exec_buffer
                        ; cmp BYTE [rcx], 0
                        ; jz => r
                        ; => l
                    )
                }
                IR::JNZ(_) => {
                    let (l, r) = lr_loop.pop().unwrap();
                    // 如果指针不为0，则向前跳转到对应的[指令处
                    dynasm! (exec_buffer
                        ; cmp BYTE [rcx], 0
                        ; jnz => l
                        ; => r
                    )
                }
            }
        }

        dynasm! (exec_buffer
            ; ret
        );

        // 调用才会分配实际内存
        let real_buffer = exec_buffer.finalize().unwrap();
        let mut brainfuck_buffer: Box<[u8]> = vec![0; 65535].into_boxed_slice();
        let brainfuck_buffer_from = brainfuck_buffer.as_mut_ptr();
        let brainfuck_buffer_to = unsafe { brainfuck_buffer_from.add(brainfuck_buffer.len()) };
        let func: fn(brainfuck_buffer_from: *mut u8, brainfuck_buffer_to: *mut u8) =
            unsafe { std::mem::transmute(real_buffer.ptr(entry_point)) };
        func(brainfuck_buffer_from, brainfuck_buffer_to);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1])?;

    let mut interpreter = Interpreter::default();
    interpreter.run(data);

    Ok(())
}
