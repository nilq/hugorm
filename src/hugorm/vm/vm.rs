use std::fmt;
use std::mem;

use colored::Colorize;

#[macro_use]
use super::*;

#[derive(Debug, PartialEq)]
pub enum Op {
    Halt   = 0x00,
    Push   = 0x01,
    Pop    = 0x02,
    Set    = 0x03,
    JmpF   = 0x04,
    Jmp    = 0x05,
    Eq     = 0x06,
    Dump   = 0x07,
    PushF  = 0x08,
    PopF   = 0x09,
    AddI   = 0x10,
    AddF   = 0x11,
    SubI   = 0x12,
    SubF   = 0x13,
    MulI   = 0x14,
    MulF   = 0x15,
    DivI   = 0x16,
    DivF   = 0x17,
    ConvIF = 0x18,
    ConvFI = 0x19,
    PushV  = 0x20,
    Call   = 0x21,
    Ret    = 0x22,
}

fn read (mem: &[u8], from: u32, size: u32) -> Vec<u8> {
    mem[from as usize .. (from + size) as usize].iter().cloned().collect()
}

pub struct VM {
    pub vars:   [u8; 262144],
    pub stack: [u8; 262144],
    pub calls: [u8; 262144],

    pub frames: Vec<u32>,

    stack_top: u32,
    vars_top:   u32,
    calls_top:  u32,
}

impl VM {
    pub fn new() -> Self {
        VM {
            vars:  [0; 262144],
            stack: [0; 262144],
            calls: [0; 262144],

            frames: vec!(0),

            stack_top: 0,
            vars_top: 0,
            calls_top: 0,
        }
    }

    pub fn exec(&mut self, bytecode: &[u8], functions: usize) -> Result<(), ()> {
        use self::Op::*;

        let mut ip: u32 = 0;

        loop {
            // print!("\nexec: {:?}", unsafe { mem::transmute::<u8, Op>(bytecode[ip as usize]) });
            match unsafe { mem::transmute::<u8, Op>(bytecode[ip as usize]) } {
                Halt => {
                    break
                },

                Push => {
                    ip += 1;
            
                    let size = (bytecode[ip as usize] as i8).abs() as u8;
            
                    ip += 1;
            
                    let value = &read(&bytecode, ip as u32, size as u32);
                    ip += size as u32;
            
                    push!(value => self.stack, [self.stack_top; size as u32]);
                },
          
                Pop => {
                    ip += 1;
          
                    let size = bytecode[ip as usize];
          
                    ip += 1;
          
                    let address = from_bytes!(&bytecode[ip as usize .. ip as usize + 4] => u32);
          
                    ip += 4;
          
                    let value = &read(&self.stack, self.stack_top - size as u32, size as u32);
          
                    self.stack_top -= size as u32;

                    memmove!(value => self.vars, [address + *self.frames.last().unwrap(); size as u32]);

                    let new_top = (address + self.frames.last().unwrap() + size as u32);
                    if self.vars_top < new_top {
                        println!("NEW TOP: {} vs {}", new_top, address);
                        self.vars_top = new_top
                    }
                },

                PushF => {
                    ip += 1;
                    
                    self.frames.push(self.vars_top)
                },
        
                PopF => {
                    ip += 1;
                    self.vars_top = self.frames.pop().unwrap();
                },

                AddI => {
                    ip += 1;
          
                    let size = (bytecode[ip as usize] as i8).abs();
          
                    ip += 1;

                    let b = pop!([&self.stack, self.stack_top] => i32);
                    let a = pop!([&self.stack, self.stack_top] => i32);

                    push!(&to_bytes!(a.wrapping_add(b) => i32) => self.stack, [self.stack_top; size as u32]);
                },

                AddF => {
                    ip += 1;
          
                    let size = (bytecode[ip as usize] as i8).abs();
          
                    ip += 1;

                    let b = pop!([&self.stack, self.stack_top] => f64);
                    let a = pop!([&self.stack, self.stack_top] => f64);

                    push!(&to_bytes!(a + b => f64) => self.stack, [self.stack_top; size as u32]);
                },

                SubI => {
                    ip += 1;
          
                    let size = (bytecode[ip as usize] as i8).abs();
          
                    ip += 1;

                    let b = pop!([&self.stack, self.stack_top] => i32);
                    let a = pop!([&self.stack, self.stack_top] => i32);

                    push!(&to_bytes!(a.wrapping_sub(b) => i32) => self.stack, [self.stack_top; size as u32]);
                },

                SubF => {
                    ip += 1;
          
                    let size = (bytecode[ip as usize] as i8).abs();
          
                    ip += 1;

                    let b = pop!([&self.stack, self.stack_top] => f64);
                    let a = pop!([&self.stack, self.stack_top] => f64);

                    push!(&to_bytes!(a - b => f64) => self.stack, [self.stack_top; size as u32]);
                },

                MulI => {
                    ip += 1;
          
                    let size = (bytecode[ip as usize] as i8).abs();
          
                    ip += 1;

                    let b = pop!([&self.stack, self.stack_top] => i32);
                    let a = pop!([&self.stack, self.stack_top] => i32);

                    push!(&to_bytes!(a.wrapping_mul(b) => i32) => self.stack, [self.stack_top; size as u32]);
                },

                DivI => {
                    ip += 1;
          
                    let size = (bytecode[ip as usize] as i8).abs();
          
                    ip += 1;

                    let b = pop!([&self.stack, self.stack_top] => i32);
                    let a = pop!([&self.stack, self.stack_top] => i32);

                    push!(&to_bytes!(a.wrapping_div(b) => i32) => self.stack, [self.stack_top; size as u32]);
                },

                DivF => {
                    ip += 1;
          
                    let size = (bytecode[ip as usize] as i8).abs();
          
                    ip += 1;

                    let b = pop!([&self.stack, self.stack_top] => f64);
                    let a = pop!([&self.stack, self.stack_top] => f64);

                    push!(&to_bytes!(a / b => f64) => self.stack, [self.stack_top; size as u32]);
                },

                Dump => {
                    ip += 1;
            
                    let size = bytecode[ip as usize];
            
                    ip += 1;
            
                    self.stack_top -= size as u32;
                },

                // Int to Float ; size_from size_to ; convif i8 f32
                ConvIF => {
                    ip += 1;
        
                    let mut size_from = bytecode[ip as usize] as i8;
        
                    let is_signed = size_from < 0;
        
                    if is_signed {
                    size_from = -size_from 
                    };
        
                    ip += 1;
        
                    let size_to = bytecode[ip as usize];
        
                    ip += 1;
        
                    self.stack_top += 16 - size_from as u32;
        
                    let value = from_bytes!(&read(&self.stack, self.stack_top - 16, 16) => u128);
        
                    self.stack_top -= 16;
        
                    let new_value = if is_signed {
                        value as i128 as f64
                    } else {
                        value as f64
                    };
        
                    let converted_size = mem::size_of::<f64>() as u32;
        
                    memmove!(&to_bytes!(new_value => f64) => self.stack, [self.stack_top; converted_size]);      
        
                    self.stack_top += converted_size
                },
        
                ConvFI => {
                    ip += 1;
        
                    let mut size_from = bytecode[ip as usize] as i8;
        
                    let is_signed = size_from < 0;
        
                    if is_signed {
                    size_from = -size_from 
                    };
        
                    ip += 1;
        
                    let size_to = bytecode[ip as usize];
        
                    ip += 1;
        
                    let value = from_bytes!(&read(&self.stack, self.stack_top - size_from as u32, size_from as u32) => u32);
        
                    let new_value = if is_signed {
                        value as i128 as f64
                    } else {
                        value as f64
                    };

                    let converted_size = mem::size_of::<f64>() as u32;

                    memmove!(&to_bytes!(new_value => u64)[0 .. size_to as usize] => self.stack, [self.stack_top; size_to as u32]);

                    self.stack_top += converted_size
                },

                PushV => {
                    ip += 1;
            
                    let scope_offset = bytecode[ip as usize];
            
                    ip += 1;
            
                    let size = bytecode[ip as usize];
            
                    ip += 1;

                    let address = from_bytes!(&bytecode[ip as usize .. ip as usize + 4] => u32) + self.frames[self.frames.len() - scope_offset as usize];
            
                    ip += 4;
            
                    let value = &read(&self.vars, address, size as u32);
            
                    push!(value => self.stack, [self.stack_top; size as u32]);
                },

                Call => {
                    ip += 1;
                    let address = pop!([&self.stack, self.stack_top] => u32) + functions as u32;    // the address of the called function
                    push!((&to_bytes!(ip => u32)) => self.calls, [self.calls_top; 4]); // address for the `ret` to return to
            
                    ip = address
                },

                Ret => {
                    ip = pop!([&self.calls, self.calls_top] => u32);
                },

                Jmp => {          
                    ip += 1;
            
                    let address = from_bytes!(&bytecode[ip as usize .. ip as usize + 4] => u32);

                    println!("\njumping from {} to {} :: out of {}\n", ip, address, bytecode.len());

                    ip = address
                },

                JmpF => {          
                    ip += 1;
            
                    if !pop!([&self.stack, self.stack_top] => bool) {
                        let address = from_bytes!(&bytecode[ip as usize .. ip as usize + 4] => u32);
            
                        ip = address
                    } else {
                        ip += 4
                    }
                },

                _ => ()
            }
        }

        Ok(())
    }
}