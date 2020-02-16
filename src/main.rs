extern crate colored;

mod hugorm;
use hugorm::lexer::*;
use hugorm::source::*;
use hugorm::parser::*;
use hugorm::vm::*;
use hugorm::visitor::*;

use std::mem;

fn main() {
    let test = r#"
fn main():
    let brr = 10
    let brr2 = 10
    30

let d = main()
    "#;

    let source = Source::from("<test.hu>", test.lines().map(|x| x.into()).collect::<Vec<String>>());
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
        Ok(ref ast) => {
            println!("{:#?}", ast);
            println!("\n--------------\n");

            let mut visitor = Visitor::new(&source, &ast);

            match visitor.visit() {
                Ok(_) => {
                    println!("\n--------------\n");

                    let mut compiler = Compiler::new(&mut visitor);
                    let mut vm = VM::new();

                    compiler.compile(&ast);

                    let mut bytecode = compiler.bytecode;

                    vm.exec(bytecode.as_slice(), compiler.functions_i);

                    println!("\n\n------- STACK -------");
                    println!("{:?}", &vm.stack[0 .. 48]);

                    println!("\n\n------- HEAP -------");
                    println!("{:?}", &vm.vars[0 .. 48]);

                    // let mut b: [u8; mem::size_of::<f64>()] = [0; mem::size_of::<f64>()];
                    // b.copy_from_slice(&vm.vars[0 .. 8]);

                    // println!("\n--------------\n");
                    // println!("human readable top: {:?}", unsafe { mem::transmute::<_, f64>(b) })
                },
                _ => (),
            }
        },

        _ => ()
    }
}
