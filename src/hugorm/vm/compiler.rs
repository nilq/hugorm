#[macro_use]
use super::*;
use std::mem;
use colored::Colorize;
use std::collections::HashMap;

pub struct Compiler<'a> {
    pub bytecode: Vec<u8>,
    pub functions: Vec<u8>, // function time
    pub functions_i: usize,

    pub visitor: &'a mut Visitor<'a>,

    pub function_ast: HashMap<String, Statement>,

    frame_index: usize,
    in_func: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(visitor: &'a mut Visitor<'a>) -> Self {
        Compiler {
            bytecode: Vec::new(),
            functions: Vec::new(),
            functions_i: 0,
            visitor,

            function_ast: HashMap::new(),

            in_func: false,

            frame_index: 0,
        }
    }

    pub fn compile(&mut self, ast: &Vec<Statement>) -> Result<(), ()> {
        self.visitor.symtab.cache_mode = true;

        for statement in ast.iter() {
            self.compile_statement(statement)?
        }

        self.visitor.symtab.cache_mode = false;

        self.emit(Op::Halt);

        self.functions_i = self.bytecode.len();
        self.bytecode.extend(self.functions.clone());

        Ok(())
    }

    pub fn compile_statement(&mut self, statement: &Statement) -> Result<(), ()> {
        use self::StatementNode::*;

        match statement.node {
            Expression(ref expression) => self.compile_expression(expression)?,
            Declaration(ref left, ref right) => self.compile_ass(left, right)?,
            Assignment(ref left, ref right) => {
                if let ExpressionNode::Identifier(ref name) = left.node {
                    self.compile_ass(name, &Some(right.to_owned()))?
                } else {
                    panic!("hmm")
                }
            },

            Function(ref name, ..) => {
                self.function_ast.insert(name.to_owned(), statement.clone());
            },

            Return(ref expr) => {
                if let Some(expr) = expr {
                    self.compile_expression(expr)?;
                }

                self.emit(Op::PopF);

                let last = self.visitor.symtab.last.clone();
                self.visitor.symtab.cached_frames.push(last);

                self.emit(Op::Ret);
            }

            _ => (),
        }

        Ok(())
    }

    pub fn compile_function(&mut self, function: &Statement, params_t: Vec<Type>) -> Result<(), ()> {
        if let StatementNode::Function(ref name, ref params, ref body) = function.node {
            use self::StatementNode::*;

            // self.emit(Op::Jmp);

            // let jump = self.bytecode.len();        // reference, for changing tmp address
            // self.emit_bytes(&to_bytes!(0 => u32)); // tmp address

            let function_address = &to_bytes!(self.functions.len() as u32 => u32);

            for (i, param) in params.iter().enumerate() {
                let t = params_t[i].clone();

                self.emit(Op::Pop); // pop the arg
                self.emit_byte(t.size().abs() as u8);
            
                let offset = t.meta.unwrap().0;

                let address = &to_bytes!(offset => u32);
                self.emit_bytes(address);
            }

            self.in_func = true;

            self.visitor.symtab.pop_cache();

            let info = format!("\n\n<function :: {}>\n", name).blue();
            print!("{}", info);

            self.emit(Op::PushF);

            let mut found_early_return = false;

            for statement in body.iter() {
                if let StatementNode::Return(..) = statement.node {
                    found_early_return = true
                }

                self.compile_statement(statement)?
            }

            if !found_early_return {
                self.emit(Op::PopF);

                let last = self.visitor.symtab.last.clone();
                self.visitor.symtab.cached_frames.push(last);
    
                self.emit(Op::Ret);                
            }

            println!();

            self.in_func = false;

            // let address = to_bytes!(self.functions.len() as u32 => u32);

            // for (i, byte) in address.iter().enumerate() {
            //     self.bytecode[jump + i] = *byte
            // }

            self.emit(Op::Push);
            self.emit_byte(4);
            self.emit_bytes(function_address);

            // declaration time B)

            let offset = self.visitor.symtab.fetch_cache(name).unwrap().meta.unwrap().0;
            let address = &to_bytes!(offset => u32);

            self.emit(Op::Pop);
            self.emit_byte(4);
            self.emit_bytes(address);

            let info = format!("\tfunc: {} @ {}", name, offset).blue();
            print!("{}", info);

            Ok(())
        } else {
            unreachable!()
        }
    }

    pub fn compile_expression(&mut self, expression: &Expression) -> Result<(), ()> {
        use self::ExpressionNode::*;

        match expression.node {
            Int(ref n) => {
                self.emit(Op::Push);
                self.emit_byte(mem::size_of::<i32>() as u8);
                self.emit_bytes(
                    unsafe {
                        &mem::transmute::<i32, [u8; mem::size_of::<i32>()]>(*n)
                    }
                )
            },

            Float(ref n) => {
                self.emit(Op::Push);
                self.emit_byte(mem::size_of::<f64>() as u8);
                self.emit_bytes(
                    unsafe {
                        &mem::transmute::<f64, [u8; mem::size_of::<f64>()]>(*n)
                    }
                )
            },

            Str(ref n) => {
                self.emit(Op::Push);
                self.emit_byte(n.len() as u8);
                self.emit_bytes(n.as_bytes());
            },
    
            Bool(ref n) => {
                self.emit(Op::Push);
                self.emit_byte(mem::size_of::<u8>() as u8);
                self.emit_byte(*n as u8)
            },

            Identifier(ref n) => {
                let t = self.visitor.symtab.fetch_cache(n).unwrap();
                let (offset, depth) = t.meta.unwrap();
                let size = t.size();

                self.emit(Op::PushV);
                self.emit_byte(depth as u8);
                self.emit_byte(size as u8);
                self.emit_bytes(&to_bytes!(offset => u32));
            },

            Binary(ref left, ref op, ref right) => {
                use self::Operator::*;

                match *op {
                    Add => {
                        self.compile_expression(&left.clone())?;
                        self.compile_expression(right)?;

                        match self.visitor.type_expression(left)?.node {
                            TypeNode::Int => {
                                self.emit(Op::AddI);
                                self.emit_byte(mem::size_of::<i32>() as u8);
                            }
                            TypeNode::Float => {
                                self.emit(Op::AddF);
                                self.emit_byte(mem::size_of::<f64>() as u8);
                            }
                            _ => (), // grrrr
                        }
                    },

                    Sub => {
                        self.compile_expression(left)?;
                        self.compile_expression(right)?;

                        match left.node {
                            ExpressionNode::Int(_) => {
                                self.emit(Op::SubI);
                                self.emit_byte(mem::size_of::<i32>() as u8);
                            }
                            ExpressionNode::Float(_) => {
                                self.emit(Op::SubF);
                                self.emit_byte(mem::size_of::<f64>() as u8);
                            }
                            _ => (), // grrrr
                        }
                    },

                    Mul => {
                        self.compile_expression(left)?;
                        self.compile_expression(right)?;

                        match left.node {
                            ExpressionNode::Int(_) => {
                                self.emit(Op::MulI);
                                self.emit_byte(mem::size_of::<i32>() as u8);
                            }
                            ExpressionNode::Float(_) => {
                                self.emit(Op::MulF);
                                self.emit_byte(mem::size_of::<f64>() as u8);
                            }
                            _ => (), // grrrr
                        }
                    },

                    Div => {
                        self.compile_expression(left)?;
                        self.compile_expression(right)?;

                        self.emit(Op::DivF); // lel
                        self.emit_byte(mem::size_of::<f64>() as u8);
                    },

                    _ => (),
                }
            },

            Call(ref caller, ref args) => {
                // TODO: check for double
                let mut params = Vec::new();

                for arg in args.iter() {
                    params.push(self.visitor.type_expression(arg)?)
                }

                if let ExpressionNode::Identifier(ref n) = caller.node {
                    let func = self.function_ast[n].clone(); // haha

                    self.compile_function(&func, params)?;
                }

                self.compile_expression(caller)?;
                self.emit(Op::Call);
            }

            _ => (),
        }

        Ok(())
    }

    fn compile_ass(&mut self, left: &String, right: &Option<Expression>) -> Result<(), ()> {
        use self::TypeNode::*;

        self.compile_expression(&right.clone().unwrap())?;

        let right_t = self.visitor.type_expression(right.as_ref().unwrap())?;

        let offset = self.visitor.symtab.fetch_cache(left).unwrap().meta.unwrap().0;
        let address = &to_bytes!(offset => u32);

        self.emit(Op::Pop);
        self.emit_byte(right_t.size().abs() as u8);
        self.emit_bytes(address);

        let info = format!("\tvar: {} @ {}", left, offset).magenta();
        print!("{}", info);

        Ok(())
    }

    fn emit(&mut self, code: Op) {
        if self.in_func {
            let info = format!("\n{:?}", code).blue();
            print!("{}", info);
            self.functions.push(code as u8)
        } else {
            print!("\n{:?}", code);
            self.bytecode.push(code as u8)
        }
    }

    fn emit_byte(&mut self, byte: u8) {
        if self.in_func {
            let info = format!("\t{:?} ", byte as i8).blue();
            print!("{}", info);
            self.functions.push(byte)
        } else {
            print!("\t{:?} ", byte as i8);
            self.bytecode.push(byte)
        }
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        if self.in_func {
            let info = format!("\t{:?} ", bytes.iter().map(|x| *x as i8).collect::<Vec<i8>>()).blue();
            print!("{}", info);
            self.functions.extend(bytes)
        } else {
            print!("\t{:?} ", bytes.iter().map(|x| *x as i8).collect::<Vec<i8>>());
            self.bytecode.extend(bytes)
        }
    }
}