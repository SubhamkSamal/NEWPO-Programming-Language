mod token;
mod lexer;
mod ast;
mod parser;
mod value;
mod interpreter;

use std::env;
use std::fs;
use std::process;

use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Newpo: No source file provided.");
        eprintln!("Usage: newpo <filename.po>");
        process::exit(1);
    }

    let filename = &args[1];

    if !filename.ends_with(".po") {
        eprintln!("Newpo: Source files must have a .po extension.");
        process::exit(1);
    }

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("Newpo: Could not read file: {}", error);
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);

    let statements = match parser.parse() {
        Ok(statements) => statements,
        Err(error) => {
            eprintln!("Newpo Parser Error: {}", error);
            process::exit(1);
        }
    };

    let mut interpreter = Interpreter::new();

    if let Err(error) = interpreter.interpret(&statements) {
        eprintln!("Newpo Runtime Error: {}", error);
        process::exit(1);
    }
}
