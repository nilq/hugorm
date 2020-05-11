extern crate colored;
extern crate rustyline;
extern crate zub;

mod hugorm;
use hugorm::lexer::*;
use hugorm::source::*;
use hugorm::parser::*;
use hugorm::visitor::*;

use zub::vm::*;

use colored::Colorize;

fn main() {
    let test = r#"
fun a(b):
    fun a(c):
        return c

    return a(b)

print(a("hey hey"))
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
            // println!("{:#?}", ast);
            // println!("\n--------------\n");

            let mut visitor = Visitor::new(&source, &ast);

            visitor.set_global("print", TypeNode::Func(1));

            match visitor.visit() {
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

                    println!("{:#?}", ir);

                    vm.exec(&ir, true);
                },
                _ => (),
            }
        },

        _ => ()
    }
}
