use std::collections::HashMap;
use std::fmt::{self, Display, Formatter, Write};
use std::rc::Rc;

use super::super::error::Response::*;
use std::cell::RefCell;

use super::*;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::mem;

use zub::ir::{ IrBuilder, ExprNode, Binding, IrFunctionBody, IrFunction, Expr, TypeInfo, BinaryOp, Literal };

pub type VarPos = Binding;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeNode {
    Int,
    Float,
    Bool,
    Str,
    Any,
    Char,
    Nil,
    Func(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeMode {
    Undeclared,
    Immutable,
    Regular,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub node: TypeNode,
    pub mode: TypeMode,
    pub meta: Option<VarPos>
}

impl Type {
    pub fn new(node: TypeNode, mode: TypeMode) -> Self {
        Self {
            node,
            mode,
            meta: None,
        }
    }

    pub fn from(node: TypeNode) -> Type {
        Type::new(node, TypeMode::Regular)
    }

    pub fn set_offset(&mut self, offset: VarPos) {
        self.meta = Some(offset)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inside {
    Loop,
    Function,
    Nothing,
}

pub struct Visitor<'a> {
    pub source: &'a Source,
    pub function_depth: usize,
    pub depth: usize,
    pub inside: Vec<Inside>,
    pub symtab: SymTab,
    pub builder: IrBuilder,
    pub repl: bool,
}

impl<'a> Visitor<'a> {
    pub fn new(source: &'a Source) -> Self {
        Visitor {
            source,
            symtab: SymTab::new(),
            inside: Vec::new(),
            depth: 0,
            function_depth: 0,
            builder: IrBuilder::new(),
            repl: false,
        }
    }

    pub fn from(source: &'a Source, symtab: SymTab) -> Self {
        Visitor {
            source,
            symtab,
            inside: Vec::new(),
            depth: 0,
            function_depth: 0,
            builder: IrBuilder::new(),
            repl: false
        }
    }

    pub fn set_global(&mut self, name: &str, t: TypeNode) {
        self.assign(name.to_string(), Type::from(t))
    }

    pub fn visit(&mut self, ast: &Vec<Statement>) -> Result<(), ()> {
        self.symtab.push();

        for statement in ast.iter() {
            self.visit_statement(&statement)?
        }

        self.symtab.pop();

        Ok(())
    }

    pub fn build(&self) -> Vec<ExprNode> {
        self.builder.build()
    }

    pub fn visit_statement(&mut self, statement: &Statement) -> Result<(), ()> {
        use self::StatementNode::*;

        let position = statement.pos.clone();

        match statement.node {
            Expression(ref expr) => {
                self.visit_expression(expr)?;

                let ir = self.compile_expression(expr)?;
                self.builder.emit(ir);

                Ok(())
            }
            Declaration(..) => self.visit_variable(&statement.node, &statement.pos),
            Assignment(..) => self.visit_ass(&statement.node, &statement.pos),

            Return(ref value) => {
                if self.inside.contains(&Inside::Function) {
                    let ret = if let Some(ref expression) = *value {
                        self.visit_expression(expression)?;

                        Some(self.compile_expression(expression)?)
                    } else {
                        None
                    };

                    self.builder.ret(ret);

                    Ok(())
                } else {
                    return Err(response!(
                        Wrong("can't return outside of function"),
                        self.source.file,
                        statement.pos
                    ));
                }
            },

            Function(ref name, ref params, ref body) => {
                let mut t = Type::from(TypeNode::Func(params.len()));

                let mut binding = Binding::local(name, self.depth, self.function_depth);

                t.set_offset(binding.clone());

                self.assign(name.to_owned(), t);

                let old_current = self.builder.clone();
                self.builder = IrBuilder::new();

                self.function_depth += 1;
                self.push_scope();
                self.inside.push(Inside::Function);

                for param in params.iter() {
                    let mut t = Type::from(TypeNode::Any);
                    t.set_offset(Binding::local(param.as_str(), self.depth, self.function_depth));

                    self.assign(param.clone(), t)
                }

                for statement in body.iter() {
                    self.visit_statement(statement)?;
                }


                self.inside.pop();
                self.pop_scope();
                self.function_depth -= 1;

                self.builder.ret(None);

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

                self.builder.emit(Expr::Function(ir_func).node(TypeInfo::nil()));
                
                Ok(())
            },

            Interface(_, ref content) => {
                for fun in content.iter() {
                    self.visit_statement(fun)?
                }

                Ok(())
            }

            While(ref cond, ref body) => {
                self.visit_expression(cond)?;

                if TypeNode::Bool == self.type_expression(cond)?.node {
                    let cond = self.compile_expression(cond)?;

                    let old_current = self.builder.clone();
                    self.builder = IrBuilder::new();

                    self.push_scope();
                    self.depth -= 1; // brother bruh

                    self.inside.push(Inside::Loop);

                    for statement in body.iter() {
                        self.visit_statement(statement)?;
                    }

                    self.inside.pop();

                    self.depth += 1; // hehe
                    self.pop_scope();


                    let body = Expr::Block(self.builder.build()).node(TypeInfo::nil());

                    self.builder = old_current;

                    self.builder.emit(
                        Expr::While(cond, body).node(TypeInfo::nil())
                    );

                    Ok(())
                } else {
                    return Err(response!(
                        Wrong("can't have non-boolean condition"),
                        self.source.file,
                        position
                    ))
                }
            }

            If(ref cond, ref body, ref else_) => {
                self.visit_expression(cond)?;

                if TypeNode::Bool == self.type_expression(cond)?.node {
                    let cond = self.compile_expression(cond)?;

                    let old_current = self.builder.clone();
                    self.builder = IrBuilder::new();

                    self.push_scope();
                    self.depth -= 1; // brother bruh

                    for statement in body.iter() {
                        self.visit_statement(statement)?;
                    }

                    self.depth += 1; // brother bruh again
                    self.pop_scope();

                    let body = Expr::Block(self.builder.build()).node(TypeInfo::nil());

                    self.builder = old_current;

                    let mut else_blocks = Expr::Literal(Literal::Nil);

                    for (i, els) in else_.iter().enumerate() {
                        let old_current = self.builder.clone();
                        self.builder = IrBuilder::new();

                        self.push_scope();

                        if let Some(ref cond) = els.0 {
                            let pos = cond.pos.clone();

                            let elif = Statement::new(
                                StatementNode::If(cond.clone(), els.1.clone(), else_[i + 1 ..].to_vec()),
                                pos
                            );

                            self.visit_statement(&elif)?;

                            self.pop_scope();

                            break // 9000 IQ

                        } else {
                            for statement in els.1.iter() {
                                self.visit_statement(statement)?;
                            }
                        }

                        self.pop_scope();

                        let body = self.builder.build();

                        self.builder = old_current;

                        else_blocks = Expr::Block(body);
                    }

                    self.builder.emit(Expr::If(cond, body, Some(else_blocks.node(TypeInfo::nil()))).node(TypeInfo::nil() ));

                    Ok(())

                } else {
                    return Err(response!(
                        Wrong("can't have non-boolean condition"),
                        self.source.file,
                        position
                    ))
                }
            }

            Break => {
                if self.inside.contains(&Inside::Loop) {
                    self.builder.break_();

                    Ok(())
                } else {
                    return Err(response!(
                        Wrong("you need a loop to break out of here"),
                        self.source.file,
                        position
                    ))
                }
            }

            Const(..) => return Err(response!(
                Wrong("constants are not implemented yet"),
                self.source.file,
                position
            )),

            ConstFunction(ref fun) => return Err(response!(
                Wrong("constants are not implemented yet"),
                self.source.file,
                position
            )),

            _ => {
                return Err(response!(
                    Wrong("what the actual fuck"),
                    self.source.file,
                    position
                ))
            }
        }
    }

    fn compile_expression(&mut self, expression: &Expression) -> Result<ExprNode, ()> {
        use self::ExpressionNode::*;

        let result = match expression.node {
            Float(ref n) => self.builder.number(*n),
            Int(ref n) => self.builder.number(*n as f64),
            Str(ref s) => self.builder.string(s),
            Bool(ref b) => self.builder.bool(*b),

            Identifier(ref n) =>  {
                if ["print", "dis"].contains(&n.as_str()) {
                    self.builder.var(Binding::global("print"))
                } else {
                    if let Some(binding) = self.symtab.fetch(n) {
                        let mut binding = binding.meta.unwrap();

                        binding = Binding::local(n, self.depth, binding.function_depth);

                        self.builder.var(binding)
                    } else {
                        return Err(response!(
                            Wrong(format!("no such variable `{}`", n)),
                            self.source.file,
                            expression.pos
                        ));
                    }
                }
            }

            Call(ref callee, ref args) => {
                let mut args_ir = Vec::new();

                for arg in args.iter() {
                    args_ir.push(self.compile_expression(arg)?)
                }

                let callee_ir = self.compile_expression(callee)?;

                self.builder.call(callee_ir, args_ir, None)
            }

            Binary(ref left, ref op, ref right) => {
                let left_ir = self.compile_expression(left)?;


                let right_ir = if op == &Index {
                    if let ExpressionNode::Identifier(ref n) = right.node {
                        Expr::Literal(
                            Literal::String(n.clone())
                        ).node(TypeInfo::nil())

                    } else {
                        unreachable!()
                    }
                } else {
                    self.compile_expression(right)?
                };

                use self::Operator::*;

                let op_ir = match op {
                    Add   => BinaryOp::Add,
                    Sub   => BinaryOp::Sub,
                    Mul   => BinaryOp::Mul,
                    Div   => BinaryOp::Div,
                    Mod   => BinaryOp::Rem,
                    And   => BinaryOp::And,
                    Or    => BinaryOp::Or,
                    Eq    => BinaryOp::Equal,
                    NEq   => BinaryOp::NEqual,
                    Lt    => BinaryOp::Lt,
                    LtEq  => BinaryOp::LtEqual,
                    Gt    => BinaryOp::Gt,
                    GtEq  => BinaryOp::GtEqual,
                    Index => BinaryOp::Index,
                    Pow   => BinaryOp::Pow, 
                    _ => todo!()
                };

                self.builder.binary(left_ir, op_ir, right_ir)
            }

            Array(ref content) => {
                let mut cont_ir = Vec::new();

                for element in content.iter() {
                    cont_ir.push(self.compile_expression(element)?)
                }

                self.builder.list(cont_ir)
            }

            Dict(ref content) => {
                let mut keys = Vec::new();
                let mut vals = Vec::new();

                for (key, val) in content.iter() {
                    keys.push(
                        Expr::Literal(
                            Literal::String(key.clone())
                        ).node(TypeInfo::nil())
                    );
                    vals.push(self.compile_expression(val)?);
                }

                self.builder.dict(keys, vals)
            }

            ref c => todo!("{:#?}", c),
        };

        Ok(result)
    }

    pub fn visit_expression(&mut self, expression: &Expression) -> Result<(), ()> {
        use self::ExpressionNode::*;

        match expression.node {
            Call(ref caller, ref args) => {
                let caller_t = self.type_expression(caller)?.node;

                if let TypeNode::Func(ref params) = caller_t {
                    if *params != args.len() {
                        return Err(response!(
                            Wrong(format!("wrong amount of arguments, expected {} but got {}", params, args.len())),
                            self.source.file,
                            caller.pos
                        ))
                    }
                } else {
                    if caller_t != TypeNode::Any {
                        return Err(response!(
                            Wrong(format!("trying to call non-function: `{:?}`", caller_t)),
                            self.source.file,
                            caller.pos
                        ))
                    }
                }

                Ok(())
            },

            Array(ref content) => {
                for element in content.iter() {
                    self.visit_expression(element)?
                }

                Ok(())
            },

            Dict(ref content) => {
                for (_, value) in content.iter() {
                    self.visit_expression(value)?
                }

                Ok(())
            },

            _ => Ok(())
        }
    }

    pub fn type_expression(&mut self, expression: &Expression) -> Result<Type, ()> {
        use self::ExpressionNode::*;

        let t = match expression.node {
            Str(_) => Type::from(TypeNode::Str),
            Bool(_) => Type::from(TypeNode::Bool),
            Int(_) => Type::from(TypeNode::Int),
            Float(_) => Type::from(TypeNode::Float),
            Binary(ref left, ref op, ref right) => {
                use self::Operator::*;

                if op == &Index {
                    // TODO
                    return Ok(Type::from(TypeNode::Any))
                }

                match (
                    self.type_expression(left)?.node,
                    op,
                    self.type_expression(right)?.node,
                ) {
                    (ref a, ref op, ref b) => match **op {
                        Add | Sub | Mul | Div | Mod => {
                            if [a, b] != [&TypeNode::Nil, &TypeNode::Nil] {
                                // real hack here
                                if a == b {
                                    match a {
                                        TypeNode::Float | TypeNode::Int => match b {
                                            TypeNode::Float | TypeNode::Int => {
                                                Type::from(a.clone())
                                            }

                                            _ => {
                                                return Err(response!(
                                                    Wrong(format!(
                                                        "can't perform operation `{:?} {} {:?}`",
                                                        a, op, b
                                                    )),
                                                    self.source.file,
                                                    expression.pos
                                                ))
                                            }
                                        },

                                        _ => {
                                            return Err(response!(
                                                Wrong(format!(
                                                    "can't perform operation `{:?} {} {:?}`",
                                                    a, op, b
                                                )),
                                                self.source.file,
                                                expression.pos
                                            ))
                                        }
                                    }
                                } else {
                                    return Err(response!(
                                        Wrong(format!(
                                            "can't perform operation `{:?} {} {:?}`",
                                            a, op, b
                                        )),
                                        self.source.file,
                                        expression.pos
                                    ));
                                }
                            } else {
                                return Err(response!(
                                    Wrong(format!("can't perform operation `{:?} {} {:?}`", a, op, b)),
                                    self.source.file,
                                    expression.pos
                                ));
                            }
                        }

                        Pow => match a {
                            TypeNode::Float | TypeNode::Int => match b {
                                TypeNode::Float | TypeNode::Int => Type::from(a.clone()),

                                _ => {
                                    return Err(response!(
                                        Wrong(format!(
                                            "can't perform operation `{:?} {} {:?}`",
                                            a, op, b
                                        )),
                                        self.source.file,
                                        expression.pos
                                    ))
                                }
                            },

                            _ => {
                                return Err(response!(
                                    Wrong(format!("can't perform operation `{:?} {} {:?}`", a, op, b)),
                                    self.source.file,
                                    expression.pos
                                ))
                            }
                        },

                        And | Or => {
                            if a == b && *a == TypeNode::Bool {
                                Type::from(TypeNode::Bool)
                            } else {
                                return Err(response!(
                                    Wrong(format!("can't perform operation `{:?} {} {:?}`", a, op, b)),
                                    self.source.file,
                                    expression.pos
                                ));
                            }
                        }

                        Concat => {
                            if *a == TypeNode::Str {
                                match *b {
                                    _ => Type::from(TypeNode::Str),
                                }
                            } else {
                                return Err(response!(
                                    Wrong(format!("can't perform operation `{:?} {} {:?}`", a, op, b)),
                                    self.source.file,
                                    expression.pos
                                ));
                            }
                        }

                        Eq | Lt | Gt | NEq | LtEq | GtEq => {
                            if a == b {
                                Type::from(TypeNode::Bool)
                            } else {
                                return Err(response!(
                                    Wrong(format!("can't perform operation `{:?} {} {:?}`", a, op, b)),
                                    self.source.file,
                                    expression.pos
                                ));
                            }
                        }

                        _ => {
                            return Err(response!(
                                Wrong(format!("can't perform operation `{:?} {} {:?}`", a, op, b)),
                                self.source.file,
                                expression.pos
                            ))
                        }
                    },
                }
            },

            Neg(ref expr) => self.type_expression(expr)?,
            Not(_) => Type::from(TypeNode::Bool),

            Identifier(ref n) => match self.symtab.fetch(n) {
                Some(t) => t,
                None    => return Err(response!(
                    Wrong(format!("no such variable `{}`", n)),
                    self.source.file,
                    expression.pos
                ))
            },

            Call(ref caller, ref args) => Type::from(TypeNode::Any),

            _ => Type::from(TypeNode::Nil),
        };

        Ok(t)
    }

    fn visit_variable(&mut self, variable: &StatementNode, pos: &Pos) -> Result<(), ()> {
        use self::ExpressionNode::*;

        if let &StatementNode::Declaration(ref name, ref right) = variable {
            if name.as_str().chars().last().unwrap() == '-' {
                response!(
                    Weird("kebab-case at identifier end is not cool"),
                    self.source.file,
                    pos
                )
            }

            if right.is_none() {
                let mut t = Type::from(TypeNode::Nil);

                t.set_offset(Binding::local(name.as_str(), self.depth, self.function_depth));
                
                self.assign(name.to_owned(), t);
                let right_ir = self.builder.number(0.0);
                let binding = Binding::local(name, self.depth, self.function_depth);

                self.builder.bind(binding, right_ir);

            } else {
                let binding = if let Some(ref t) = self.symtab.fetch(name) {
                    t.meta.clone().unwrap()
                } else {
                    Binding::local(name.as_str(), self.depth, self.function_depth)
                };

                let mut t = self.type_expression(right.as_ref().unwrap())?;

                t.set_offset(binding.clone());

                self.assign(name.to_owned(), t);

                let right_ir = self.compile_expression(&right.clone().unwrap())?;

                self.builder.bind(binding, right_ir);
            }
        }

        Ok(())
    }

    fn visit_ass(&mut self, ass: &StatementNode, pos: &Pos) -> Result<(), ()> {
        use self::ExpressionNode::*;

        if let &StatementNode::Assignment(ref name, ref right) = ass {            
            if let ExpressionNode::Identifier(ref name) = name.node {

                if let Some(left_t) = self.symtab.fetch(name) {
                    let binding = left_t.meta.unwrap().clone();
    
                    let mut t = self.type_expression(&right)?;
                    t.set_offset(binding);
    
                    self.assign(name.to_owned(), t)
                } else {
                    return Err(response!(
                        Wrong(format!("can't assign non-existent `{}`", name)),
                        self.source.file,
                        pos
                    ))
                }
            }

            self.visit_expression(right)?;

            let left_ir = self.compile_expression(name)?;
            let right_ir = self.compile_expression(right)?;

            self.builder.mutate(left_ir, right_ir)
        }

        Ok(())
    }

    fn assign_str(&mut self, name: &str, t: Type) {
        self.symtab.assign_str(name, t)
    }

    fn assign(&mut self, name: String, t: Type) {
        self.symtab.assign(name, t)
    }

    fn push_scope(&mut self) {
        self.symtab.push();
        
        self.depth += 1
    }

    fn pop_scope(&mut self) {
        self.symtab.pop();

        self.depth -= 1
    }
}