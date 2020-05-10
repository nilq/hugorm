use std::collections::HashMap;
use std::fmt::{self, Display, Formatter, Write};
use std::rc::Rc;

use super::super::error::Response::*;

use super::*;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::mem;

pub type VarPos = (usize, usize);

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
    pub ast: &'a Vec<Statement>,
    pub function_depth: usize,
    pub depth: usize,
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
            depth: 0,
            function_depth: 0,
        }
    }

    pub fn from(source: &'a Source, ast: &'a Vec<Statement>, symtab: SymTab) -> Self {
        Visitor {
            source,
            ast,
            symtab,
            inside: Vec::new(),
            depth: 0,
            function_depth: 0,
        }
    }

    pub fn visit(&mut self) -> Result<(), ()> {
        self.symtab.push(); // can't push_scope, cause it increases depth - that's wack, def don't want that

        for statement in self.ast.iter() {
            self.visit_statement(&statement)?
        }

        self.symtab.pop(); // cleaning up. don't pop_scope

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
                let mut t = Type::from(TypeNode::Func(params.len()));

                println!("set func {} @ {} {}", name, self.depth, self.function_depth);

                t.set_offset((self.depth, self.function_depth));
                self.assign(name.to_owned(), t);


                self.function_depth += 1;
                self.push_scope();
                self.inside.push(Inside::Function);

                for param in params.iter() {
                    let mut t = Type::from(TypeNode::Any);
                    t.set_offset((self.depth, self.function_depth));

                    self.assign(param.clone(), t)
                }

                for statement in body.iter() {
                    self.visit_statement(statement)?
                }

                self.inside.pop();
                self.pop_scope();
                self.function_depth -= 1;

                Ok(())
            },

            Interface(_, ref content) => {
                for fun in content.iter() {
                    self.visit_statement(fun)?
                }

                Ok(())
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
                    return Err(response!(
                        Wrong(format!("trying to call non-function: `{:?}`", caller_t)),
                        self.source.file,
                        caller.pos
                    ))
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
                self.assign(name.to_owned(), Type::from(TypeNode::Nil))
            } else {
                let (depth, function_depth) = if let Some(ref t) = self.symtab.fetch(name) {
                    t.meta.unwrap()
                } else {
                    (self.depth, self.function_depth)
                };

                let mut t = self.type_expression(right.as_ref().unwrap())?;

                println!("set {} @ depth({}) funcs({})", name, depth, function_depth);
                t.set_offset((depth, function_depth));

                self.assign(name.to_owned(), t);
            }
        }

        Ok(())
    }

    fn visit_ass(&mut self, ass: &StatementNode, pos: &Pos) -> Result<(), ()> {
        use self::ExpressionNode::*;

        if let &StatementNode::Assignment(ref name, ref right) = ass {            
            if let ExpressionNode::Identifier(ref name) = name.node {

                if let Some(left_t) = self.symtab.fetch(name) {
                    let (offset, depth) = left_t.meta.unwrap().clone();
    
                    let mut t = self.type_expression(&right)?;
                    t.set_offset((offset, depth));
    
    
                    self.assign(name.to_owned(), t);
                } else {
                    return Err(response!(
                        Wrong(format!("can't assign non-existent `{}`", name)),
                        self.source.file,
                        pos
                    ))
                }
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
        
        self.depth += 1
    }

    fn pop_scope(&mut self) {
        self.symtab.pop();

        self.depth -= 1
    }
}