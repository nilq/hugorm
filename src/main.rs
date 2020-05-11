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

use colored::Colorize;

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn test() {
    let test = r#"
fun a(b):
    fun ø'(c):
        return c

    return ø'(b)

    "#;

    let source = Source::from("<test.hug>", test.lines().map(|x| x.into()).collect::<Vec<String>>());
    let lexer = Lexer::default(test.chars().collect(), &source);

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
            println!("{:#?}", ast);
            println!("\n--------------\n");

            let mut visitor = Visitor::new(&source);

            visitor.set_global("print", TypeNode::Func(1));

            match visitor.visit(&ast) {
                Ok(_) => {
                    println!("{}", "We're good".green());

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
    println!("{}", "Hugorm REPL | Interactive gangster terminal");

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
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
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
                    Ok(new_ast) => {
                        match visitor.visit(&new_ast) {
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
                println!("CTRL-C");
                break
            },

            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },

            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}

fn main() {
    repl()
}