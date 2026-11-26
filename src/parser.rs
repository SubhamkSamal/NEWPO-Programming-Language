
use crate::ast::{BinaryOp, Expr, Stmt, UnaryOp};
use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    // Returns the current token without consuming it.
    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }

    // Consumes and returns the current token.
    fn advance(&mut self) -> Token {
        let token = self.current().clone();

        if !matches!(token, Token::Eof) {
            self.position += 1;
        }

        token
    }

    // Skips statement separators.
    fn skip_newlines(&mut self) {
        while matches!(
            self.current(),
            Token::Newline | Token::Semicolon
        ) {
            self.advance();
        }
    }

    // Skips only newlines inside delimited expressions.
    // Semicolons remain invalid inside collections and argument lists.
    fn skip_collection_newlines(&mut self) {
        while matches!(self.current(), Token::Newline) {
            self.advance();
        }
    }

    // Checks whether the current token matches the expected token.
    fn consume(
        &mut self,
        expected: Token,
        message: &str,
    ) -> Result<(), String> {
        if *self.current() == expected {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "{}. Found {:?}",
                message,
                self.current()
            ))
        }
    }

    // Parses the complete program.
    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();

        self.skip_newlines();

        while !matches!(self.current(), Token::Eof) {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok(statements)
    }

    // Parses one statement.
    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match self.current() {
            Token::When => self.parse_when(),

            Token::Loop => self.parse_loop(),

            Token::Function => self.parse_function(),

            Token::Return => {
                self.advance();

                if matches!(
                    self.current(),
                    Token::Newline
                        | Token::Semicolon
                        | Token::RightBrace
                        | Token::Eof
                ) {
                    Ok(Stmt::Return(None))
                } else {
                    Ok(Stmt::Return(Some(
                        self.parse_expression(0)?
                    )))
                }
            }

            Token::Break => {
                self.advance();
                Ok(Stmt::Break)
            }

            Token::Continue => {
                self.advance();
                Ok(Stmt::Continue)
            }

            Token::Import => {
                self.advance();

                match self.advance() {
                    Token::String(path) => {
                        Ok(Stmt::Import(path))
                    }

                    _ => Err(
                        "Expected a string path after import".to_string()
                    ),
                }
            }

            Token::Identifier(name) => {
                let name = name.clone();

                let is_assignment = matches!(
                    self.tokens.get(self.position + 1),
                    Some(Token::Assign)
                );

                if is_assignment {
                    self.advance();
                    self.advance();

                    let value = self.parse_expression(0)?;

                    Ok(Stmt::Assign {
                        name,
                        value,
                    })
                } else {
                    let expression = self.parse_expression(0)?;

                    Ok(Stmt::Expression(expression))
                }
            }

            _ => {
                let expression = self.parse_expression(0)?;

                Ok(Stmt::Expression(expression))
            }
        }
    }

    // Parses:
    // when condition {
    //     statements
    // } otherwise {
    //     statements
    // }
    fn parse_when(&mut self) -> Result<Stmt, String> {
        self.advance();

        let condition = self.parse_expression(0)?;

        self.consume(
            Token::LeftBrace,
            "Expected '{' after when condition",
        )?;

        let body = self.parse_block()?;

        self.skip_newlines();

        let otherwise = if matches!(
            self.current(),
            Token::Otherwise
        ) {
            self.advance();

            self.consume(
                Token::LeftBrace,
                "Expected '{' after otherwise",
            )?;

            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::When {
            condition,
            body,
            otherwise,
        })
    }

    // Parses:
    // loop condition {
    //     statements
    // }
    fn parse_loop(&mut self) -> Result<Stmt, String> {
        self.advance();

        let condition = self.parse_expression(0)?;

        self.consume(
            Token::LeftBrace,
            "Expected '{' after loop condition",
        )?;

        let body = self.parse_block()?;

        Ok(Stmt::Loop {
            condition,
            body,
        })
    }

    // Parses function declarations.
    fn parse_function(&mut self) -> Result<Stmt, String> {
        self.advance();

        let name = match self.advance() {
            Token::Identifier(name) => name,

            _ => {
                return Err(
                    "Expected function name".to_string()
                );
            }
        };

        self.consume(
            Token::LeftParen,
            "Expected '(' after function name",
        )?;

        let mut parameters = Vec::new();

        self.skip_collection_newlines();

        if !matches!(self.current(), Token::RightParen) {
            loop {
                self.skip_collection_newlines();

                match self.advance() {
                    Token::Identifier(parameter) => {
                        parameters.push(parameter);
                    }

                    _ => {
                        return Err(
                            "Expected parameter name".to_string()
                        );
                    }
                }

                self.skip_collection_newlines();

                if matches!(self.current(), Token::Comma) {
                    self.advance();
                    self.skip_collection_newlines();

                    if matches!(
                        self.current(),
                        Token::RightParen
                    ) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        self.skip_collection_newlines();

        self.consume(
            Token::RightParen,
            "Expected ')' after parameters",
        )?;

        self.consume(
            Token::LeftBrace,
            "Expected '{' before function body",
        )?;

        let body = self.parse_block()?;

        Ok(Stmt::Function {
            name,
            parameters,
            body,
        })
    }

    // Parses statements inside a block.
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();

        self.skip_newlines();

        while !matches!(
            self.current(),
            Token::RightBrace | Token::Eof
        ) {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        self.consume(
            Token::RightBrace,
            "Expected '}' to close block",
        )?;

        Ok(statements)
    }

    // Parses expressions using operator precedence.
    fn parse_expression(
        &mut self,
        min_precedence: u8,
    ) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;

        loop {
            // Conditional expression:
            // condition ? value_if_true : value_if_false
            if matches!(self.current(), Token::Question) {
                if min_precedence > 0 {
                    break;
                }

                self.advance();

                let then_branch = self.parse_expression(0)?;

                self.consume(
                    Token::Colon,
                    "Expected ':' in conditional expression",
                )?;

                let else_branch = self.parse_expression(0)?;

                left = Expr::Conditional {
                    condition: Box::new(left),
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                };

                continue;
            }

            let (operator, precedence) = match self.current() {
                Token::Or => (BinaryOp::Or, 1),

                Token::And => (BinaryOp::And, 2),

                Token::Equal => (BinaryOp::Equal, 3),
                Token::NotEqual => (BinaryOp::NotEqual, 3),

                Token::Greater => (BinaryOp::Greater, 3),
                Token::Less => (BinaryOp::Less, 3),
                Token::GreaterEqual => (BinaryOp::GreaterEqual, 3),
                Token::LessEqual => (BinaryOp::LessEqual, 3),

                Token::Plus => (BinaryOp::Add, 4),
                Token::Minus => (BinaryOp::Subtract, 4),

                Token::Star => (BinaryOp::Multiply, 5),
                Token::Slash => (BinaryOp::Divide, 5),

                _ => break,
            };

            if precedence < min_precedence {
                break;
            }

            self.advance();

            let right =
                self.parse_expression(precedence + 1)?;

            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // Parses unary operators.
    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.current() {
            Token::Minus => {
                self.advance();

                Ok(Expr::Unary {
                    operator: UnaryOp::Negate,
                    operand: Box::new(self.parse_unary()?),
                })
            }

            Token::Not | Token::Bang => {
                self.advance();

                Ok(Expr::Unary {
                    operator: UnaryOp::Not,
                    operand: Box::new(self.parse_unary()?),
                })
            }

            _ => self.parse_postfix(),
        }
    }

    // Parses function calls and collection indexing.
    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expression = self.parse_primary()?;

        loop {
            match self.current() {
                // Function call:
                // function_name(arguments)
                Token::LeftParen => {
                    self.advance();

                    let mut arguments = Vec::new();

                    self.skip_collection_newlines();

                    if !matches!(
                        self.current(),
                        Token::RightParen
                    ) {
                        loop {
                            self.skip_collection_newlines();

                            arguments.push(
                                self.parse_expression(0)?
                            );

                            self.skip_collection_newlines();

                            if matches!(
                                self.current(),
                                Token::Comma
                            ) {
                                self.advance();
                                self.skip_collection_newlines();

                                if matches!(
                                    self.current(),
                                    Token::RightParen
                                ) {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }

                    self.skip_collection_newlines();

                    self.consume(
                        Token::RightParen,
                        "Expected ')' after function arguments",
                    )?;

                    expression = Expr::Call {
                        callee: Box::new(expression),
                        arguments,
                    };
                }

                // Collection indexing:
                // collection[index]
                Token::LeftBracket => {
                    self.advance();

                    self.skip_collection_newlines();

                    let index = self.parse_expression(0)?;

                    self.skip_collection_newlines();

                    self.consume(
                        Token::RightBracket,
                        "Expected ']' after collection index",
                    )?;

                    expression = Expr::Index {
                        collection: Box::new(expression),
                        index: Box::new(index),
                    };
                }

                _ => break,
            }
        }

        Ok(expression)
    }

    // Parses primitive values and collection literals.
    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Token::Number(value) => {
                Ok(Expr::Number(value))
            }

            Token::String(value) => {
                Ok(Expr::String(value))
            }

            Token::True => {
                Ok(Expr::Boolean(true))
            }

            Token::False => {
                Ok(Expr::Boolean(false))
            }

            Token::Null => {
                Ok(Expr::Null)
            }

            Token::Identifier(name) => {
                Ok(Expr::Variable(name))
            }

            // Parenthesized expression.
            Token::LeftParen => {
                let expression = self.parse_expression(0)?;

                self.consume(
                    Token::RightParen,
                    "Expected ')' after expression",
                )?;

                Ok(expression)
            }

            // List literal:
            // [10, 20, 30]
            // Also supports multiple lines.
            Token::LeftBracket => {
                let mut elements = Vec::new();

                self.skip_collection_newlines();

                if !matches!(
                    self.current(),
                    Token::RightBracket
                ) {
                    loop {
                        self.skip_collection_newlines();

                        elements.push(
                            self.parse_expression(0)?
                        );

                        self.skip_collection_newlines();

                        if matches!(
                            self.current(),
                            Token::Comma
                        ) {
                            self.advance();
                            self.skip_collection_newlines();

                            if matches!(
                                self.current(),
                                Token::RightBracket
                            ) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }

                self.skip_collection_newlines();

                self.consume(
                    Token::RightBracket,
                    "Expected ']' after list elements",
                )?;

                Ok(Expr::List(elements))
            }

            // Dictionary literal:
            // {
            //     "name": "Alex",
            //     "age": 20
            // }
            // Also supports multiple lines.
            Token::LeftBrace => {
                let mut entries = Vec::new();

                self.skip_collection_newlines();

                if !matches!(
                    self.current(),
                    Token::RightBrace
                ) {
                    loop {
                        self.skip_collection_newlines();

                        let key = self.parse_expression(0)?;

                        self.skip_collection_newlines();

                        self.consume(
                            Token::Colon,
                            "Expected ':' between dictionary key and value",
                        )?;

                        self.skip_collection_newlines();

                        let value = self.parse_expression(0)?;

                        entries.push((key, value));

                        self.skip_collection_newlines();

                        if matches!(
                            self.current(),
                            Token::Comma
                        ) {
                            self.advance();
                            self.skip_collection_newlines();

                            if matches!(
                                self.current(),
                                Token::RightBrace
                            ) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }

                self.skip_collection_newlines();

                self.consume(
                    Token::RightBrace,
                    "Expected '}' after dictionary entries",
                )?;

                Ok(Expr::Dictionary(entries))
            }

            token => {
                let position =
                    self.position.saturating_sub(1);

                Err(format!(
                    "Unexpected token in expression: {:?} at token position {}",
                    token,
                    position
                ))
            }
        }
    }
}