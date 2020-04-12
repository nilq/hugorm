use internment::Intern;
use broom::prelude::*;
use super::*;

use std::collections::HashMap;

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum Op {
    Int(i32),
    Float(f64),
    Bool(u8),
    Jmp(u32),
    JmpF(u32),
    Ret,
    PushLocal,
    Local(u32),
}

type Ident = Intern<String>;

struct Memory {
    heap: Heap<HeapObject>,
    locals: Vec<Rooted<HeapObject>>,
    globals: HashMap<Ident, Rooted<HeapObject>>,
}

impl Memory {
    #[inline]
    pub fn get(&self, handle: impl AsRef<Handle<HeapObject>>) -> &HeapObject {
        self.heap.get(handle).unwrap()
    }

    #[inline]
    pub fn push_local(&mut self, v: Rooted<HeapObject>) {
        self.locals.push(v)
    }

    #[inline]
    pub fn replace_local(&mut self, offset: usize, v: Rooted<HeapObject>) -> Rooted<HeapObject> {
        let len = self.locals.len();
        std::mem::replace(&mut self.locals[len - 1 - offset], v)
    }

    pub fn local(&self, offset: usize) -> &Rooted<HeapObject> {
        &self.locals[self.locals.len() - 1 - offset]
    }

    pub fn pop_local(&mut self) {
        self.locals.pop().unwrap();
    }

    pub fn global(&self, ident: Ident) -> Option<&Rooted<HeapObject>> {
        self.globals.get(&ident)
    }
}

pub struct VM {
    mem: Memory,
    stack: Vec<Handle<HeapObject>>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            mem: Memory {
                heap:    Heap::default(),
                locals:  Vec::new(),
                globals: HashMap::new(),
            },
            stack: Vec::new()
        }
    }

    fn handle(&mut self, a: HeapObject) -> Handle<HeapObject> {
        self.mem.heap.insert_temp(a)
    }
 
    pub fn exec(&mut self, ops: Vec<Op>) {
        use self::Op::*;

        let mut pc = 0u32;

        loop {
            let op = &ops[pc as usize];

            pc += 1;

            match op {
                Int(x) => {
                    let handle = self.handle(Value::from(*x).as_heap());
                    self.stack.push(handle)
                },
                Float(x) => {
                    let handle = self.handle(Value::from(*x).as_heap());
                    self.stack.push(handle)
                },
                PushLocal => {
                    let a = self.stack.pop().unwrap();
                    let local = self.mem.heap.make_rooted(a);

                    self.mem.push_local(local)
                },
                Ret => return,
                Local(o) => self.stack.push(self.mem.local(*o as usize).handle()),
                _ => ()
            }
        }
    }

    #[inline]
    pub fn visualize(&self) {
        println!("\n");
        println!("GLOBALS: {:#?}", self.mem.globals);

        println!("LOCALS:{:#?}", self.mem.locals);

        println!("STACK: {:#?}", self.stack);
    }
}