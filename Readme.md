# Newpo Programming Language

Newpo is a Python-inspired, dynamically typed programming language built using Rust. It was developed as a project to explore programming language design and interpreter implementation.

## Features

- Variables and dynamic values
- Arithmetic and logical expressions
- Conditional statements and loops
- Functions and return statements
- Lists and dictionaries
- User input and output
- File imports
- Built-in functions
- `break` and `continue`

## Example

```newpo
name := "Newpo"
say "Hello from " + name
```

## Architecture

Newpo's interpreter is organized into separate components:

- `lexer.rs` — Converts source code into tokens
- `parser.rs` — Parses tokens into an abstract syntax tree
- `ast.rs` — Defines the language's syntax structures
- `interpreter.rs` — Executes parsed programs
- `value.rs` — Represents runtime values
- `token.rs` — Defines token types
- `main.rs` — Program entry point

## Built With

- Rust

## Purpose

Newpo was created to gain practical experience with programming language design, parsing, abstract syntax trees, and interpreter development.

## Repository

[GitHub Repository](https://github.com/SubhamkSamal/NEWPO-Programming-Language.git)