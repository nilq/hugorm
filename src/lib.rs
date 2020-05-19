extern crate colored;
extern crate rustyline;
extern crate rustyline_derive;
extern crate zub;
extern crate gag;

use std::io::Read;

mod hugorm;
use hugorm::lexer::*;
use hugorm::source::*;
use hugorm::parser::*;
use hugorm::visitor::*;

use zub::vm::*;

use std::io; 
use std::path::Path;
use std::fs::File;

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
