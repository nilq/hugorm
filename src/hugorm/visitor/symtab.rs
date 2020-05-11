use std::cell::RefCell;
use std::collections::HashMap;

use std::rc::Rc;

use super::visitor::*;

#[derive(Debug, Clone)]
pub struct Frame {
    pub table: RefCell<HashMap<String, Type>>,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            table: RefCell::new(HashMap::new()),
        }
    }

    pub fn from(table: HashMap<String, Type>) -> Self {
        Frame {
            table: RefCell::new(table),
        }
    }

    pub fn get(&self, name: &String) -> Option<Type> {
        if let Some(v) = self.table.borrow().get(name) {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn assign(&mut self, name: String, t: Type) {
        self.table.borrow_mut().insert(name, t);
    }

    pub fn debug(&self) {
        println!("======= frame");
        for (name, t) in self.table.borrow().iter() {
            println!("{} = {:?}", name, t)
        }

        println!()
    }
}

#[derive(Debug, Clone)]
pub struct SymTab {
    pub stack: Vec<Frame>, // active frames
    pub cached_frames: Vec<Frame>,
    pub last: Frame,       // last frame
    pub cache_mode: bool,
    pub foreign_imports: HashMap<String, HashMap<String, Type>>,
}

impl SymTab {
    pub fn new() -> Self {
        SymTab {
            stack: vec![Frame::new()],
            last: Frame::new(),
            cached_frames: Vec::new(),
            cache_mode: false,
            foreign_imports: HashMap::new(),
        }
    }

    pub fn from(table: HashMap<String, Type>) -> Self {
        SymTab {
            stack: vec![Frame::from(table)],
            last: Frame::new(),
            cached_frames: Vec::new(),
            cache_mode: false,
            foreign_imports: HashMap::new(),
        }
    }

    pub fn assign(&mut self, name: String, t: Type) {
        self.current_frame_mut().assign(name, t)
    }

    pub fn assign_str(&mut self, name: &str, t: Type) {
        self.current_frame_mut().assign(name.to_string(), t)
    }

    pub fn fetch(&self, name: &String) -> Option<Type> {
        if self.cache_mode {
            return self.fetch_cache(name)
        }

        let mut offset = self.stack.len() - 1;

        loop {
            if let Some(t) = self.stack[offset].get(name) {
                return Some(t);
            } else {
                if offset == 0 {
                    return None;
                }

                offset -= 1;
            }
        }
    }

    pub fn fetch_str(&self, name: &str) -> Option<Type> {
        if self.cache_mode {
            self.fetch_cache(&name.to_string())
        } else {
            self.fetch(&name.to_string())
        }
    }

    pub fn current_frame(&self) -> &Frame {
        self.stack.last().unwrap()
    }

    pub fn current_frame_mut(&mut self) -> &mut Frame {
        self.stack.last_mut().unwrap()
    }

    pub fn put_frame(&mut self, frame: Frame) {
        self.stack.push(frame)
    }

    pub fn push(&mut self) {
        self.stack.push(Frame::new())
    }

    pub fn pop(&mut self) {
        self.last = self.stack.pop().unwrap();
        self.cached_frames.push(self.last.clone())
    }

    pub fn pop_cache(&mut self) {
        self.last = self.cached_frames.pop().unwrap()
    }

    pub fn fetch_cache(&self, name: &String) -> Option<Type> {
        let mut offset = self.cached_frames.len() - 1;

        loop {
            if let Some(t) = self.cached_frames[offset].get(name) {
                return Some(t);
            } else {

                if offset == 0 {
                    return None;
                }

                offset -= 1;
            }
        }
    }

    pub fn get_foreign_module(&self, id: &String) -> Option<&HashMap<String, Type>> {
        self.foreign_imports.get(id)
    }

    pub fn import(&mut self, id: String, origin: HashMap<String, Type>) {
        self.foreign_imports.insert(id, origin);
    }
}
