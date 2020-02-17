use std::collections::HashMap;
use std::fmt::{self, Display, Formatter, Write};
use std::rc::Rc;

use super::super::error::Response::*;

use super::*;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::mem;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeNode {
    Int,
    Float,
    Bool,
    Str,
    Any,
    Char,
    Nil,
    Func,
    Id(Rc<Expression>),
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
    pub meta: Option<(u32, u32)>
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

    pub fn set_offset(&mut self, offset: (u32, u32)) {
        self.meta = Some(offset)
    }

    pub fn size(&self) -> i8 {
        use self::TypeNode::*;

        match self.node {
            Int   => mem::size_of::<i32>() as i8,
            Float => mem::size_of::<f64>() as i8,
            Bool  => mem::size_of::<bool>() as i8,
            Char  => mem::size_of::<char>() as i8,
            Nil   => 0,
            Func  => 4,
            _     => panic!("no size yet."),
        }
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
    pub ast: &'a Vec<Statement>,
    pub offsets: Vec<u32>,
    pub depth: u32,
    pub inside: Vec<Inside>,
    pub symtab: SymTab,
}

impl<'a> Visitor<'a> {
    pub fn new(source: &'a Source, ast: &'a Vec<Statement>) -> Self {
        Visitor {
            source,
            ast,
            symtab: SymTab::new(),
            inside: Vec::new(),
            offsets: vec!(0),
            depth: 0,
        }
    }

    pub fn visit(&mut self) -> Result<(), ()> {
        self.push_scope();

        for statement in self.ast.iter() {
            self.visit_statement(&statement)?
        }

        self.pop_scope();

        Ok(())
    }

    pub fn visit_statement(&mut self, statement: &Statement) -> Result<(), ()> {
        use self::StatementNode::*;

        let position = statement.pos.clone();

        match statement.node {
            Expression(ref expr) => self.visit_expression(expr),
            Declaration(..) => self.visit_variable(&statement.node, &statement.pos),
            Assignment(..) => self.visit_ass(&statement.node, &statement.pos),

            Return(ref value) => {
                if self.inside.contains(&Inside::Function) {
                    if let Some(ref expression) = *value {
                        self.visit_expression(expression)
                    } else {
                        Ok(())
                    }
                } else {
                    return Err(response!(
                        Wrong("can't return outside of function"),
                        self.source.file,
                        statement.pos
                    ));
                }
            },

            Function(ref name, ref params, ref body) => {
                let offset = *self.offsets.last().unwrap();
                let depth  = self.depth;

                let mut t = Type::from(TypeNode::Func);

                t.set_offset((offset, depth));

                let len = self.offsets.len();
                self.offsets[len - 1] += 8 as u32;

                self.assign(name.to_owned(), t);

                self.push_scope();
                self.inside.push(Inside::Function);

                for statement in body.iter() {
                    self.visit_statement(statement);
                }

                self.inside.pop();
                self.pop_scope();

                Ok(())
            }

            _ => {
                return Err(response!(
                    Wrong("what the actual fuck"),
                    self.source.file,
                    position
                ))
            }
        }
    }

    pub fn visit_expression(&mut self, expression: &Expression) -> Result<(), ()> {
        use self::ExpressionNode::*;

        match expression.node {
            _ => Ok(())
        }
    }

    pub fn type_expression(&mut self, expression: &Expression) -> Result<Type, ()> {
        use self::ExpressionNode::*;

        let t = match expression.node {
            Str(_) => Type::from(TypeNode::Str),
            Char(_) => Type::from(TypeNode::Char),
            Bool(_) => Type::from(TypeNode::Bool),
            Int(_) => Type::from(TypeNode::Int),
            Float(_) => Type::from(TypeNode::Float),

            Call(..) => Type::from(TypeNode::Func), // TODO: lol

            Binary(ref left, ref op, ref right) => {
                use self::Operator::*;

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
                                    // TypeNode::Func(..) | TypeNode::Array(..) => {
                                    //     return Err(response!(
                                    //         Wrong(format!(
                                    //             "can't perform operation `{:?} {} {:?}`",
                                    //             a, op, b
                                    //         )),
                                    //         self.source.file,
                                    //         expression.pos
                                    //     ))
                                    // }

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
            }

            _ => Type::from(TypeNode::Nil),
        };

        Ok(t)
    }

    fn visit_variable(&mut self, variable: &StatementNode, pos: &Pos) -> Result<(), ()> {
        use self::ExpressionNode::*;

        if let &StatementNode::Declaration(ref name, ref right) = variable {
            if right.is_none() {
                self.assign(name.to_owned(), Type::from(TypeNode::Nil))
            } else {
                let (offset, depth) = if let Some(ref t) = self.symtab.fetch(name) {
                    t.meta.unwrap()
                } else {
                    let offset = *self.offsets.last().unwrap();
                    let depth  = self.depth;

                    (offset, depth)
                };

                let mut t = self.type_expression(right.as_ref().unwrap())?;

                println!("set offset {} = {}", name, offset);
                t.set_offset((offset, depth));

                let len = self.offsets.len();
                self.offsets[len - 1] += 8 as u32;

                self.assign(name.to_owned(), t);
            }
        }

        Ok(())
    }

    fn visit_ass(&mut self, ass: &StatementNode, pos: &Pos) -> Result<(), ()> {
        use self::ExpressionNode::*;

        if let &StatementNode::Assignment(ref name, ref right) = ass {            
            if let ExpressionNode::Identifier(ref name) = name.node {
                let left_t = self.symtab.fetch(name).unwrap();
                let (offset, depth) = left_t.meta.unwrap().clone();

                let mut t = self.type_expression(&right)?;
                t.set_offset((offset, depth));


                self.assign(name.to_owned(), t);
            }
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
        self.offsets.push(0);
        
        self.depth += 1
    }

    fn pop_scope(&mut self) {
        self.symtab.pop();
        self.offsets.pop();

        self.depth -= 1
    }
}