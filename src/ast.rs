
#[derive(Debug, Clone)]
pub enum Expr {
    // Primitive values
    Number(f64),
    String(String),
    Boolean(bool),
    Null,

    // Variables
    Variable(String),

    // Arithmetic, comparison and logical operations
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },

    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },

    // Conditional expression
    Conditional {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },

    // Function calls
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },

    // Collection literals
    List(Vec<Expr>),

    Dictionary(Vec<(Expr, Expr)>),

    // Collection indexing
    Index {
        collection: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,

    // Comparison
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,

    // Logical
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    // Variable assignment
    Assign {
        name: String,
        value: Expr,
    },

    // Standalone expressions
    Expression(Expr),

    // Conditional blocks
    When {
        condition: Expr,
        body: Vec<Stmt>,
        otherwise: Option<Vec<Stmt>>,
    },

    // Loops
    Loop {
        condition: Expr,
        body: Vec<Stmt>,
    },

    Break,
    Continue,

    // Functions
    Function {
        name: String,
        parameters: Vec<String>,
        body: Vec<Stmt>,
    },

    Return(Option<Expr>),

    // Imports
    Import(String),
}
