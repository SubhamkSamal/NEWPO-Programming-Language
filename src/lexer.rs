
use crate::token::Token;

pub struct Lexer {
    source: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
        }
    }

    fn current(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.position + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current();
        self.position += 1;
        ch
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.current(), Some(' ' | '\t' | '\r')) {
            self.advance();
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;

        while matches!(self.current(), Some('0'..='9')) {
            self.advance();
        }

        if self.current() == Some('.') {
            self.advance();

            while matches!(self.current(), Some('0'..='9')) {
                self.advance();
            }
        }

        let number: String = self.source[start..self.position]
            .iter()
            .collect();

        Token::Number(number.parse().unwrap_or(0.0))
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;

        while matches!(
            self.current(),
            Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_')
        ) {
            self.advance();
        }

        let identifier: String = self.source[start..self.position]
            .iter()
            .collect();

        match identifier.as_str() {
            "when" => Token::When,
            "otherwise" => Token::Otherwise,
            "loop" => Token::Loop,
            "function" => Token::Function,
            "return" => Token::Return,

            "import" => Token::Import,
            "break" => Token::Break,
            "continue" => Token::Continue,

            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,

            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,

            _ => Token::Identifier(identifier),
        }
    }

    fn read_string(&mut self) -> Token {
        self.advance();

        let mut value = String::new();

        while let Some(ch) = self.current() {
            if ch == '"' {
                self.advance();
                return Token::String(value);
            }

            if ch == '\\' {
                self.advance();

                match self.advance() {
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some(other) => {
                        value.push('\\');
                        value.push(other);
                    }
                    None => break,
                }
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Token::String(value)
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.current().is_some() {
            self.skip_whitespace();

            let token = match self.current() {
                Some('\n') => {
                    self.advance();
                    Token::Newline
                }

                Some('0'..='9') => self.read_number(),

                Some('a'..='z' | 'A'..='Z' | '_') => {
                    self.read_identifier()
                }

                Some('"') => self.read_string(),

                Some(':') if self.peek() == Some('=') => {
                    self.advance();
                    self.advance();
                    Token::Assign
                }

                Some('=') if self.peek() == Some('=') => {
                    self.advance();
                    self.advance();
                    Token::Equal
                }

                Some('!') if self.peek() == Some('=') => {
                    self.advance();
                    self.advance();
                    Token::NotEqual
                }

                Some('>') if self.peek() == Some('=') => {
                    self.advance();
                    self.advance();
                    Token::GreaterEqual
                }

                Some('<') if self.peek() == Some('=') => {
                    self.advance();
                    self.advance();
                    Token::LessEqual
                }

                Some('+') => {
                    self.advance();
                    Token::Plus
                }

                Some('-') => {
                    self.advance();
                    Token::Minus
                }

                Some('*') => {
                    self.advance();
                    Token::Star
                }

                Some('/') => {
                    self.advance();
                    Token::Slash
                }

                Some('>') => {
                    self.advance();
                    Token::Greater
                }

                Some('<') => {
                    self.advance();
                    Token::Less
                }

                Some('?') => {
                    self.advance();
                    Token::Question
                }

                Some(':') => {
                    self.advance();
                    Token::Colon
                }

                Some('!') => {
                    self.advance();
                    Token::Bang
                }

                Some('(') => {
                    self.advance();
                    Token::LeftParen
                }

                Some(')') => {
                    self.advance();
                    Token::RightParen
                }

                Some('{') => {
                    self.advance();
                    Token::LeftBrace
                }

                Some('}') => {
                    self.advance();
                    Token::RightBrace
                }

                Some('[') => {
                    self.advance();
                    Token::LeftBracket
                }

                Some(']') => {
                    self.advance();
                    Token::RightBracket
                }

                Some(',') => {
                    self.advance();
                    Token::Comma
                }

                Some(';') => {
                    self.advance();
                    Token::Semicolon
                }

                Some(_) => {
                    self.advance();
                    continue;
                }

                None => break,
            };

            tokens.push(token);
        }

        tokens.push(Token::Eof);
        tokens
    }
}