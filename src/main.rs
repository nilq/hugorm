extern crate colored;
extern crate rustyline;
extern crate broom;
#[macro_use]
extern crate nanbox;
extern crate internment;
extern crate cli_table;

mod hugorm;
use hugorm::lexer::*;
use hugorm::source::*;
use hugorm::parser::*;
use hugorm::visitor::*;
use hugorm::vm::*;

fn main() {
    let test = r#"
let a = 10
let b = 20

a
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
                    let program = compile(&ast);
                    let mut vm       = VM::new();

                    vm.exec(program);

                    vm.visualize()
                },
                _ => (),
            }
        },

        _ => ()
    }
}
