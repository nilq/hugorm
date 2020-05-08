extern crate colored;
extern crate rustyline;
extern crate broom;
#[macro_use]
extern crate nanbox;
extern crate internment;
extern crate fnv;

mod hugorm;
use hugorm::lexer::*;
use hugorm::source::*;
use hugorm::parser::*;
use hugorm::visitor::*;

use colored::Colorize;

fn main() {
    let test = r#"
let aaa = {
    fruit: "hey",
    tis: "semand",
}

interface YesBoi:
    fun ølle-brød(self):
        return r"hey\n"

let snake = {} with YesBoi
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
        Ok(ref ast) => {
            println!("{:#?}", ast);
            println!("\n--------------\n");

            let mut visitor = Visitor::new(&source, &ast);

            match visitor.visit() {
                Ok(_) => {
                    println!("{}", "We're good".green())
                },
                _ => (),
            }
        },

        _ => ()
    }
}
