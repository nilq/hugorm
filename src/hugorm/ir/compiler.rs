use zub::ir::*;
use zub::vm::*;

use super::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Compiler<'a> {
    builder: IrBuilder,
    visitor: &'a mut Visitor<'a>
}

impl<'a> Compiler<'a> {
    pub fn new(visitor: &'a mut Visitor<'a>) -> Self {
        Compiler {
            builder: IrBuilder::new(),
            visitor
        }
    }

    pub fn compile(&mut self, ast: Vec<Statement>) -> Result<(), ()> {
        self.visitor.symtab.cache_mode = true;

        for s in ast.iter() {
            self.compile_statement(s)?
        }

        self.visitor.symtab.cache_mode = false;

        Ok(())
    }

    fn compile_statement(&mut self, statement: &Statement) -> Result<(), ()> {
        use self::StatementNode::*;

        match statement.node {
            Declaration(ref name, ref right) => {
                let right_ir = if let Some(right) = right {
                    self.compile_expression(right)?
                } else {
                    self.builder.number(0.0) // TODO
                };

                let offset = self.visitor.symtab.fetch_cache(name).unwrap().meta.unwrap();
                let binding = Binding::local(name, offset.0, offset.1);

                self.builder.bind(binding.clone(), right_ir.clone());

                // let var = self.builder.var(binding);

                // self.builder.mutate(var, right_ir)
            }

            Assignment(ref left, ref right) => {
                let left_ir  = self.compile_expression(left)?;
                let right_ir = self.compile_expression(right)?;

                self.builder.mutate(left_ir, right_ir);
            },

            Function(ref name, ref params, ref body) => {
                let offset = self.visitor.symtab.fetch_cache(name).unwrap().meta.unwrap();
                let binding = Binding::local(name, offset.0, offset.1);

                let old_current = self.builder.clone();
                self.builder = IrBuilder::new();

                self.visitor.symtab.pop_cache();

                for s in body.iter() {
                    self.compile_statement(s)?;
                }

                self.builder.ret(None);
                
                let last = self.visitor.symtab.last.clone();
                self.visitor.symtab.cached_frames.push(last);

                let body = self.builder.build();

                self.builder = old_current;

                let func_body = IrFunctionBody {
                    params: params.iter().cloned().map(|x|
                        Binding::local(x.as_str(), binding.depth.unwrap_or(0) + 1, binding.function_depth + 1)).collect::<Vec<Binding>>(),
                    method: false,
                    inner: body
                };

                let ir_func = IrFunction {
                    var: binding,
                    body: Rc::new(RefCell::new(func_body))
                };

                self.builder.emit(Expr::Function(ir_func).node(TypeInfo::nil()))
            },

            Return(ref value) => {
                let value_ir = if let Some(value) = value {
                    Some(self.compile_expression(value)?)
                } else {
                    None
                };

                self.builder.ret(value_ir)
            },

            Expression(ref expr) => {
                let ir = self.compile_expression(expr)?;
                self.builder.emit(ir)
            },

            _ => ()
        }

        Ok(())
    }

    fn compile_expression(&mut self, expression: &Expression) -> Result<ExprNode, ()> {
        use self::ExpressionNode::*;

        let result = match expression.node {
            Float(ref n) => self.builder.number(*n),
            Identifier(ref n) =>  {
                if n == "print" {
                    self.builder.var(Binding::global("print"))
                } else {
                    let t = self.visitor.symtab.fetch_cache(n).unwrap();
                    let (depth, function_depth) = t.meta.unwrap();

                    self.builder.var(Binding::local(n, depth, function_depth))
                }
            }
            Call(ref callee, ref args) => {
                let mut args_ir = Vec::new();

                for arg in args.iter() {
                    args_ir.push(self.compile_expression(arg)?)
                }

                let callee_ir = self.compile_expression(callee)?;

                self.builder.call(callee_ir, args_ir, None)
            },

            ref c => todo!("{:#?}", c),
        };

        Ok(result)
    }

    pub fn build(self) -> Vec<ExprNode> {
        self.builder.build()
    }
}