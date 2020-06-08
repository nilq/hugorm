extern crate colored;
extern crate rustyline;
extern crate rustyline_derive;
extern crate zub;
#[cfg(unix)]
extern crate gag;
extern crate statrs;

#[cfg(unix)]
use gag::BufferRedirect;
use std::io::Read;

mod hugorm;
use crate::hugorm::lexer::*;
use crate::hugorm::source::*;
use crate::hugorm::parser::*;
use crate::hugorm::visitor::*;
use crate::hugorm::prelude::math;

use zub::vm::*;
use zub::compiler::*;
use zub::ir::*;

use std::io;
use std::rc::Rc;

use colored::Colorize;

use std::collections::HashSet;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::{hint::Hinter, Context};
use rustyline_derive::{Completer, Helper, Highlighter, Validator};

use std::io::prelude::*;
use std::path::Path;
use std::fs::File;
use std::fs::metadata;

fn run(path: &str, content: &str, root: String) {
    let source = Source::from(path, content.lines().map(|x| x.into()).collect::<Vec<String>>());
    let lexer = Lexer::default(content.chars().collect(), &source);

    let mut tokens = Vec::new();

    for token_res in lexer {
        if let Ok(token) = token_res {
            tokens.push(token)
        } else {
            return
        }
    }

    let mut parser = Parser::new(tokens, &source);

    match parser.parse() {
        Ok(ast) => {
            let mut visitor = Visitor::new(&source, root);

            visitor.set_global("print", TypeNode::Func(1));
            visitor.set_global("input", TypeNode::Func(0));
            visitor.set_global("len", TypeNode::Func(1));

            match visitor.visit(&ast) {
                Ok(_) => {
                    visitor.symtab.pop(); // gotta cachce root scope

                    fn print(heap: &mut Heap<Object>, args: &[Value]) -> Value {
                        println!("{}", args[1].with_heap(heap));
                        Value::nil()
                    }

                    fn prompt(heap: &mut Heap<Object>, args: &[Value]) -> Value {
                        let mut input = String::new();

                        match io::stdin().read_line(&mut input) {
                            Ok(n) => {
                                Value::object(heap.insert_temp(Object::String(input)))
                            }

                            Err(error) => {
                                println!("error: {}", error);
                                Value::nil()
                            },
                        }
                    }

                    fn len(heap: &mut Heap<Object>, args: &[Value]) -> Value {
                        if let Variant::Obj(handle) = args[1].decode() {
                            if let Object::List(ref list) = unsafe { heap.get_unchecked(handle) } {
                                Value::float(list.content.len() as f64)
                            } else {
                                Value::nil()
                            }
                        } else {
                            Value::nil()
                        }
                    }

                    let mut vm = VM::new();
                    vm.add_native("print", print, 1);
                    vm.add_native("len", len, 1);

                    let ir = visitor.build();

                    vm.exec(&ir, false);
                },
                _ => (),
            }
        },

        _ => ()
    }
}

#[derive(Completer, Helper, Validator, Highlighter)]
struct HugHinter {
    hints: HashSet<String>
}

impl Hinter for HugHinter {
    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }

        self.hints
            .iter()
            .filter_map(|hint| {
                if pos > 0 && hint.starts_with(&line[..pos]) {
                    Some(format!("{}", hint[pos..].to_owned()))
                } else {
                    None
                }
            })
            .next()
    }
}

fn hugorm_hints() -> HashSet<String> {
    let mut set = HashSet::new();

    set.insert("fun".to_string());
    set.insert("print".to_string());
    set.insert("if".to_string());
    set.insert("elif".to_string());
    set.insert("else".to_string());
    set.insert("let".to_string());

    set
}

#[cfg(unix)]
fn repl(root: String) {
    let hinter = HugHinter {
        hints: hugorm_hints()
    };

    let mut rl = Editor::<HugHinter>::new();
    rl.set_helper(Some(hinter));

    let header = format!("{} {} {}", "Hugorm REPL".bold(), "|".green(), "Interactive gangster terminal".yellow().bold());
    println!("{}", header);
    println!("{}", "-------------------------------------------".green());


    let source = Source::from("<repl>", Vec::new());

    fn print(heap: &mut Heap<Object>, args: &[Value]) -> Value {
        println!("{}", args[1].with_heap(heap));
        Value::nil()
    }

    fn len(heap: &mut Heap<Object>, args: &[Value]) -> Value {
        if let Variant::Obj(handle) = args[1].decode() {
            if let Object::List(ref list) = unsafe { heap.get_unchecked(handle) } {
                Value::float(list.content.len() as f64)
            } else {
                Value::nil()
            }
        } else {
            Value::nil()
        }
    }

    let mut vm = VM::new();
    vm.add_native("print", print, 1);
    vm.add_native("len", len, 1);

    let mut visitor = Visitor::new(&source, root);

    visitor.set_global("print", TypeNode::Func(1));
    visitor.set_global("len", TypeNode::Func(1));

    let mut last_len = 0usize;

    let caret_normal = format!("{}", ">> ".green());
    let caret_buffer = ".. ".to_string();
    let mut caret = caret_normal.clone();

    let mut in_buffer = false;
    let mut debug = false;

    let mut line_buffer = String::new(); // for multiline stuff

    loop {
        let readline = rl.readline(caret.as_str());

        match readline {
            Ok(line) => {
                if line.len() == 0 {
                    if in_buffer {
                        in_buffer = false;

                        caret = caret_normal.clone();
                    } else {
                        continue
                    }
                }

                let mut line = line;

                if line.len() > 0 && line.trim().chars().last().unwrap() == ':' || in_buffer {
                    line_buffer.push_str(&line);
                    line_buffer.push('\n');

                    caret = caret_buffer.clone();

                    in_buffer = true;

                    continue
                } else if line_buffer.len() > 0 {
                    line = format!("{}\n{}", line_buffer, line);
                    line_buffer = String::new()
                }


                match line.replace(" ", "").as_str() {
                    "@math" => {
                        math::include_math(&mut visitor, &mut vm);
                        continue
                    },

                    "@debug" => {
                        debug = true;
                        continue
                    },

                    _ => ()
                }

                rl.add_history_entry(line.as_str());

                let source = Source::from("<repl>", line.lines().map(|x| x.into()).collect::<Vec<String>>());
                let lexer = Lexer::default(line.chars().collect(), &source);

                let mut tokens = Vec::new();

                for token_res in lexer {
                    if let Ok(token) = token_res {
                        tokens.push(token)
                    } else {
                        return
                    }
                }

                let mut parser = Parser::new(tokens, &source);

                match parser.parse() {
                    Ok(ast) => {
                        let mut repl_ast = Vec::new();

                        if ast.len() == 1 {
                            if let StatementNode::Expression(ref expr) = ast[0].node {
                                let pos = ast[0].pos.clone();

                                repl_ast.push(
                                    Statement::new(
                                        StatementNode::Declaration(
                                            "$".to_string(), // to capture print return no fucks
                                            Some(
                                                Expression::new(
                                                    ExpressionNode::Call(
                                                        Rc::new(
                                                            Expression::new(
                                                                ExpressionNode::Identifier(
                                                                    String::from("print")
                                                                ),
                                                                pos.clone()
                                                            )
                                                        ),
                                                        vec!(expr.clone())
                                                    ),
                                                    pos.clone()
                                                )
                                            )
                                        ),
                                        pos.clone()
                                    )
                                )
                            } else {
                                repl_ast = ast
                            }
                        } else {
                            repl_ast = ast
                        }

                        match visitor.visit(&repl_ast) {
                            Ok(_) => {
                                if debug {
                                    let ir = visitor.build();

                                    vm.exec(&ir, false);

                                    visitor.symtab.stack.push(visitor.symtab.last.clone());

                                } else {
                                    let mut buffer = BufferRedirect::stdout().unwrap();
                                    let ir = visitor.build();

                                    vm.exec(&ir, false);

                                    visitor.symtab.stack.push(visitor.symtab.last.clone());

                                    let mut output = String::new();
                                    let new_len = buffer.read_to_string(&mut output).unwrap();

                                    drop(buffer);

                                    print!("{}", &output[last_len .. new_len]);

                                    last_len = new_len;
                                }
                            }

                            _ => continue
                        }
                    },

                    _ => continue
                }
            },

            Err(ReadlineError::Interrupted) => {
                println!("Bye.");
                break
            },

            Err(ReadlineError::Eof) => {
                println!("Au revoir.");
                break
            },

            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}

fn run_file(path: &str, root: String) {
    let display = Path::new(path).display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("failed to open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();

    match file.read_to_string(&mut s) {
        Err(why) => panic!("failed to read {}: {}", display, why),
        Ok(_) => run(&path, &s, root),
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let root = Path::new(&args[0].to_string()).parent().unwrap().display().to_string();

    if args.len() == 1 {
        #[cfg(unix)]
        {
            repl(root)
        }
        #[cfg(not(unix))]
        {
            println!("Oops! REPL is not available on non-unix platforms!");
        }
    } else {
        for arg in args[1..].iter() {
            run_file(arg, root.clone())
        }
    }
}
