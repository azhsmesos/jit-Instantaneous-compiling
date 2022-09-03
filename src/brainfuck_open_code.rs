/**
* Brainfuck language definition
*/

#[derive(Debug, PartialEq)]
pub enum OpenCode {
    SHR = 0x3E,     //shift the right >
    SHL = 0x3C,     //shift the left <
    ADD = 0x2B,     // +
    SUB = 0x2D,     // -
    PUTCHAR = 0x2E, // ,
    GETCHAR = 0x2C, // .
    LB = 0x5B,      // [
    RB = 0x5D,      // ]
}

impl From<u8> for OpenCode {
    fn from(u: u8) -> Self {
        match u {
            0x3E => OpenCode::SHR,
            0x3C => OpenCode::SHL,
            0x2B => OpenCode::ADD,
            0x2D => OpenCode::SUB,
            0x2E => OpenCode::PUTCHAR,
            0x2C => OpenCode::GETCHAR,
            0x5B => OpenCode::LB,
            0x5D => OpenCode::RB,
            /*
               Ascii code ranges from 0 to 255, and if it doesn't belong to these 8 characters,
               the rest of the program will simply exit without parsing
            */
            _ => unreachable!(),
        }
    }
}

pub struct BrainfuckCode {
    pub instrs: Vec<OpenCode>,                           //指令  instruction
    pub jtable: std::collections::HashMap<usize, usize>, // 加速代码执行 ，将语言的位置写在jtable中
}

impl BrainfuckCode {
    pub fn from(data: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        let dict: Vec<u8> = vec![
            OpenCode::SHR as u8,
            OpenCode::SHL as u8,
            OpenCode::ADD as u8,
            OpenCode::SUB as u8,
            OpenCode::PUTCHAR as u8,
            OpenCode::GETCHAR as u8,
            OpenCode::LB as u8,
            OpenCode::RB as u8,
        ];
        let instrs: Vec<OpenCode> = data
            .iter()
            .filter(|x| dict.contains(x))
            .map(|x| OpenCode::from(*x))
            .collect();

        let mut jstack: Vec<usize> = Vec::new();
        let mut jtable: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for (i, e) in instrs.iter().enumerate() {
            if OpenCode::LB == *e {
                jstack.push(i);
            }
            if OpenCode::RB == *e {
                let js = jstack.pop().ok_or("pop from empty list")?;
                jtable.insert(js, i);
                jtable.insert(i, js);
            }
        }
        Ok(BrainfuckCode { instrs, jtable })
    }
}
