use std::{env, fs};

use crate::lexer::{Lexer, Token};

mod lexer;

fn main() {
    println!("Enter your string:");

    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("requires 2 arguments");

    match fs::read_to_string(path) {
        Ok(contents) => {
            let mut name = Lexer::new(&contents);

            loop {
                let element = name.next();
                println!("{:?}", element);
                if let Token::Eof = element {
                    break;
                }
            }
        }
        Err(err) => {
            panic!("Error {} reading file '{}'.", err, path);
        }
    }
}
