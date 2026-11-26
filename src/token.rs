
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Values
    Identifier(String),
    Number(f64),
    String(String),

    // Keywords
    When,
    Otherwise,
    Loop,
    Function,
    Return,

    Import,
    Break,
    Continue,

    True,
    False,
    Null,

    And,
    Or,
    Not,

    // Arithmetic operators
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /

    // Assignment and comparison operators
    Assign,       // :=
    Equal,        // ==
    NotEqual,     // !=
    Greater,      // >
    Less,         // <
    GreaterEqual, // >=
    LessEqual,    // <=

    // Conditional expression operators
    Question,     // ?
    Colon,        // :

    // Symbols
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]

    Comma,        // ,
    Semicolon,    // ;
    Bang,         // !

    // Special
    Newline,
    Eof,
}
