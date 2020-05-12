extern crate colored;
extern crate rustyline;
extern crate zub;
extern crate gag;

use gag::BufferRedirect;
use std::io::Read;

mod hugorm;
use hugorm::lexer::*;
use hugorm::source::*;
use hugorm::parser::*;
use hugorm::visitor::*;

use zub::vm::*;
use zub::compiler::*;
use zub::ir::*;

use std::rc::Rc;

use colored::Colorize;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::io::prelude::*;
use std::path::Path;
use std::fs::File;
use std::fs::metadata;

fn run(path: &str, content: &str) {
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
            let mut visitor = Visitor::new(&source);

            visitor.set_global("print", TypeNode::Func(1));

            match visitor.visit(&ast) {
                Ok(_) => {
                    visitor.symtab.pop(); // gotta cachce root scope

                    fn print(heap: &Heap<Object>, args: &[Value]) -> Value {
                        println!("{}", args[1].with_heap(heap));
                        Value::nil()
                    }

                    let mut vm = VM::new();
                    vm.add_native("print", print, 1);

                    let ir = visitor.build();

                    vm.exec(&ir, false);
                },
                _ => (),
            }
        },

        _ => ()
    }
}

fn repl() {
    let mut rl = Editor::<()>::new();
    let header = format!("{} | {}", "Hugorm REPL".bold(), "Interactive gangster terminal".yellow().bold());
    println!("{}", header);
    println!("{}", "-------------------------------------------".green());


    let source = Source::from("<repl>", Vec::new());

    fn print(heap: &Heap<Object>, args: &[Value]) -> Value {
        println!("{}", args[1].with_heap(heap));
        Value::nil()
    }

    let mut vm = VM::new();
    vm.add_native("print", print, 1);

    let mut visitor = Visitor::new(&source);

    visitor.set_global("print", TypeNode::Func(1));

    let mut last_len = 0usize;

    loop {
        let readline = rl.readline(&format!("{}", ">> ".green()));

        match readline {
            Ok(line) => {
                if line.len() == 0 {
                    continue
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
                                let mut buffer = BufferRedirect::stdout().unwrap();

                                vm.exec(&visitor.build(), false);

                                visitor.symtab.stack.push(visitor.symtab.last.clone());

                                let mut output = String::new();
                                let new_len = buffer.read_to_string(&mut output).unwrap();

                                drop(buffer);

                                print!("{}", &output[last_len .. new_len]);

                                last_len = new_len;
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

fn run_file(path: &str, root: &String) {
    let display = Path::new(path).display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("failed to open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();

    match file.read_to_string(&mut s) {
        Err(why) => panic!("failed to read {}: {}", display, why),
        Ok(_) => run(&path, &s),
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() == 1 {
        repl()
    } else {
        for arg in args[1..].iter() {
            run_file(arg, arg)
        }
    }
}