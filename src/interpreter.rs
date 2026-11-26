
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::ast::{BinaryOp, Expr, Stmt, UnaryOp};
use crate::value::{DictionaryKey, UserFunction, Value};

enum Flow {
    Normal,
    Return(Value),
    Break,
    Continue,
}

pub struct Interpreter {
    environment: HashMap<String, Value>,
    scopes: Vec<HashMap<String, Value>>,
    base_dir: PathBuf,
    imported_files: HashSet<PathBuf>,
    import_stack: HashSet<PathBuf>,
    loop_depth: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        Self::with_base_dir(
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        )
    }

    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        let mut interpreter = Self {
            environment: HashMap::new(),
            scopes: Vec::new(),
            base_dir,
            imported_files: HashSet::new(),
            import_stack: HashSet::new(),
            loop_depth: 0,
        };

        interpreter.register_builtins();
        interpreter
    }

    fn register_builtins(&mut self) {
        for name in ["say", "taker", "length", "append", "remove", "keys"] {
            self.environment.insert(
                name.to_string(),
                Value::BuiltinFunction(name.to_string()),
            );
        }
    }

    fn get_variable(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }

        self.environment.get(name).cloned()
    }

    fn assign_variable(&mut self, name: String, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name) {
                scope.insert(name, value);
                return;
            }
        }

        self.environment.insert(name, value);
    }

    pub fn interpret(
        &mut self,
        statements: &[Stmt],
    ) -> Result<(), String> {
        match self.execute_block(statements)? {
            Flow::Normal => Ok(()),
            Flow::Return(_) => Err(
                "return cannot be used outside a function".to_string()
            ),
            Flow::Break => Err(
                "break cannot be used outside a loop".to_string()
            ),
            Flow::Continue => Err(
                "continue cannot be used outside a loop".to_string()
            ),
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
    ) -> Result<Flow, String> {
        for statement in statements {
            let flow = self.execute(statement)?;

            if !matches!(&flow, Flow::Normal) {
                return Ok(flow);
            }
        }

        Ok(Flow::Normal)
    }

    fn execute(&mut self, statement: &Stmt) -> Result<Flow, String> {
        match statement {
            Stmt::Assign { name, value } => {
                let result = self.evaluate(value)?;
                self.assign_variable(name.clone(), result);
                Ok(Flow::Normal)
            }

            Stmt::Expression(expression) => {
                self.evaluate(expression)?;
                Ok(Flow::Normal)
            }

            Stmt::When {
                condition,
                body,
                otherwise,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute_block(body)
                } else if let Some(otherwise_body) = otherwise {
                    self.execute_block(otherwise_body)
                } else {
                    Ok(Flow::Normal)
                }
            }

            Stmt::Loop { condition, body } => {
                self.loop_depth += 1;

                let result: Result<Flow, String> = (|| {
                    loop {
                        if !self.evaluate(condition)?.is_truthy() {
                            break;
                        }

                        match self.execute_block(body)? {
                            Flow::Normal | Flow::Continue => {}
                            Flow::Break => break,
                            Flow::Return(value) => {
                                return Ok(Flow::Return(value));
                            }
                        }
                    }

                    Ok(Flow::Normal)
                })();

                self.loop_depth -= 1;
                result
            }

            Stmt::Break => {
                if self.loop_depth == 0 {
                    return Err("break outside a loop".to_string());
                }

                Ok(Flow::Break)
            }

            Stmt::Continue => {
                if self.loop_depth == 0 {
                    return Err("continue outside a loop".to_string());
                }

                Ok(Flow::Continue)
            }

            Stmt::Function {
                name,
                parameters,
                body,
            } => {
                let function = Value::Function(UserFunction {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    body: body.clone(),
                });

                self.assign_variable(name.clone(), function);
                Ok(Flow::Normal)
            }

            Stmt::Return(expression) => {
                let value = match expression {
                    Some(expression) => self.evaluate(expression)?,
                    None => Value::Null,
                };

                Ok(Flow::Return(value))
            }

            Stmt::Import(path) => {
                self.import_file(path)?;
                Ok(Flow::Normal)
            }
        }
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<Value, String> {
        match expression {
            Expr::Number(value) => Ok(Value::Number(*value)),

            Expr::String(value) => Ok(Value::String(value.clone())),

            Expr::Boolean(value) => Ok(Value::Boolean(*value)),

            Expr::Null => Ok(Value::Null),

            Expr::Variable(name) => {
                self.get_variable(name)
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            }

            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left)?;

                if matches!(operator, BinaryOp::And) && !left_value.is_truthy() {
                    return Ok(left_value);
                }

                if matches!(operator, BinaryOp::Or) && left_value.is_truthy() {
                    return Ok(left_value);
                }

                let right_value = self.evaluate(right)?;

                self.evaluate_binary(left_value, operator, right_value)
            }

            Expr::Unary { operator, operand } => {
                let value = self.evaluate(operand)?;

                match operator {
                    UnaryOp::Negate => match value {
                        Value::Number(number) => Ok(Value::Number(-number)),
                        _ => Err("Unary '-' requires a number".to_string()),
                    },

                    UnaryOp::Not => {
                        Ok(Value::Boolean(!value.is_truthy()))
                    }
                }
            }

            Expr::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.evaluate(then_branch)
                } else {
                    self.evaluate(else_branch)
                }
            }

            Expr::Call { callee, arguments } => {
                let function = self.evaluate(callee)?;

                let mut values = Vec::new();

                for argument in arguments {
                    values.push(self.evaluate(argument)?);
                }

                self.call_function(function, values)
            }

            Expr::List(elements) => {
                let mut values = Vec::new();

                for element in elements {
                    values.push(self.evaluate(element)?);
                }

                Ok(Value::List(values))
            }

            Expr::Dictionary(entries) => {
                let mut values = HashMap::new();

                for (key, value) in entries {
                    let key = self.evaluate(key)?.to_key()?;
                    let value = self.evaluate(value)?;

                    values.insert(key, value);
                }

                Ok(Value::Dictionary(values))
            }

            Expr::Index { collection, index } => {
                let collection = self.evaluate(collection)?;
                let index = self.evaluate(index)?;

                self.get_index(collection, index)
            }
        }
    }

    fn evaluate_binary(
        &self,
        left: Value,
        operator: &BinaryOp,
        right: Value,
    ) -> Result<Value, String> {
        match (left, operator, right) {
            (Value::Number(a), BinaryOp::Add, Value::Number(b)) =>
                Ok(Value::Number(a + b)),

            (Value::Number(a), BinaryOp::Subtract, Value::Number(b)) =>
                Ok(Value::Number(a - b)),

            (Value::Number(a), BinaryOp::Multiply, Value::Number(b)) =>
                Ok(Value::Number(a * b)),

            (Value::Number(a), BinaryOp::Divide, Value::Number(b)) => {
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }

                Ok(Value::Number(a / b))
            }

            (Value::String(a), BinaryOp::Add, Value::String(b)) =>
                Ok(Value::String(a + &b)),

            (Value::Number(a), BinaryOp::Equal, Value::Number(b)) =>
                Ok(Value::Boolean(a == b)),

            (Value::Number(a), BinaryOp::NotEqual, Value::Number(b)) =>
                Ok(Value::Boolean(a != b)),

            (Value::Number(a), BinaryOp::Greater, Value::Number(b)) =>
                Ok(Value::Boolean(a > b)),

            (Value::Number(a), BinaryOp::Less, Value::Number(b)) =>
                Ok(Value::Boolean(a < b)),

            (Value::Number(a), BinaryOp::GreaterEqual, Value::Number(b)) =>
                Ok(Value::Boolean(a >= b)),

            (Value::Number(a), BinaryOp::LessEqual, Value::Number(b)) =>
                Ok(Value::Boolean(a <= b)),

            (Value::String(a), BinaryOp::Greater, Value::String(b)) =>
                Ok(Value::Boolean(a > b)),

            (Value::String(a), BinaryOp::Less, Value::String(b)) =>
                Ok(Value::Boolean(a < b)),

            (Value::String(a), BinaryOp::GreaterEqual, Value::String(b)) =>
                Ok(Value::Boolean(a >= b)),

            (Value::String(a), BinaryOp::LessEqual, Value::String(b)) =>
                Ok(Value::Boolean(a <= b)),

            (a, BinaryOp::Equal, b) =>
                Ok(Value::Boolean(a == b)),

            (a, BinaryOp::NotEqual, b) =>
                Ok(Value::Boolean(a != b)),

            (a, BinaryOp::And, b) =>
                Ok(if a.is_truthy() { b } else { a }),

            (a, BinaryOp::Or, b) =>
                Ok(if a.is_truthy() { a } else { b }),

            _ => Err("Invalid operation between values".to_string()),
        }
    }

    fn get_index(
        &self,
        collection: Value,
        index: Value,
    ) -> Result<Value, String> {
        match collection {
            Value::List(values) => {
                let index = self.list_index(&index, values.len())?;
                Ok(values[index].clone())
            }

            Value::String(value) => {
                let characters: Vec<char> = value.chars().collect();
                let index = self.list_index(&index, characters.len())?;

                Ok(Value::String(characters[index].to_string()))
            }

            Value::Dictionary(values) => {
                let key = index.to_key()?;

                values.get(&key)
                    .cloned()
                    .ok_or_else(|| "Dictionary key not found".to_string())
            }

            _ => Err("Value cannot be indexed".to_string()),
        }
    }

    fn list_index(
        &self,
        index: &Value,
        length: usize,
    ) -> Result<usize, String> {
        let number = match index {
            Value::Number(number)
                if number.fract() == 0.0 && *number >= 0.0 =>
            {
                *number as usize
            }

            _ => return Err("Collection index must be a non-negative integer".to_string()),
        };

        if number >= length {
            return Err("Collection index out of bounds".to_string());
        }

        Ok(number)
    }

    fn call_function(
        &mut self,
        function: Value,
        arguments: Vec<Value>,
    ) -> Result<Value, String> {
        match function {
            Value::BuiltinFunction(name) => {
                self.call_builtin(&name, arguments)
            }

            Value::Function(function) => {
                if arguments.len() != function.parameters.len() {
                    return Err(format!(
                        "{} expects {} arguments, received {}",
                        function.name,
                        function.parameters.len(),
                        arguments.len()
                    ));
                }

                let mut local_scope = HashMap::new();

                for (parameter, value) in
                    function.parameters.iter().zip(arguments)
                {
                    local_scope.insert(parameter.clone(), value);
                }

                self.scopes.push(local_scope);

                let result = self.execute_block(&function.body);

                self.scopes.pop();

                match result? {
                    Flow::Normal => Ok(Value::Null),
                    Flow::Return(value) => Ok(value),
                    Flow::Break => Err("break cannot escape a function".to_string()),
                    Flow::Continue => Err("continue cannot escape a function".to_string()),
                }
            }

            _ => Err("Value is not callable".to_string()),
        }
    }

    fn call_builtin(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
    ) -> Result<Value, String> {
        match name {
            "say" => {
                let output: Vec<String> = arguments
                    .iter()
                    .map(|value| value.to_string())
                    .collect();

                println!("{}", output.join(" "));
                Ok(Value::Null)
            }

            "taker" => {
                if arguments.len() != 1 {
                    return Err("taker() expects one prompt argument".to_string());
                }

                let prompt = arguments[0].to_string();

                print!("{}", prompt);
                io::stdout()
                    .flush()
                    .map_err(|error| error.to_string())?;

                let mut input = String::new();

                io::stdin()
                    .read_line(&mut input)
                    .map_err(|error| error.to_string())?;

                let input = input.trim();

                if input == "true" {
                    return Ok(Value::Boolean(true));
                }

                if input == "false" {
                    return Ok(Value::Boolean(false));
                }

                if input == "null" {
                    return Ok(Value::Null);
                }

                if let Ok(number) = input.parse::<f64>() {
                    return Ok(Value::Number(number));
                }

                Ok(Value::String(input.to_string()))
            }

            "length" => {
                if arguments.len() != 1 {
                    return Err("length() expects one argument".to_string());
                }

                let length = match &arguments[0] {
                    Value::List(values) => values.len(),
                    Value::Dictionary(values) => values.len(),
                    Value::String(value) => value.chars().count(),
                    _ => return Err("length() requires a collection or string".to_string()),
                };

                Ok(Value::Number(length as f64))
            }

            "append" => {
                if arguments.len() != 2 {
                    return Err("append() expects two arguments".to_string());
                }

                let mut values = match &arguments[0] {
                    Value::List(values) => values.clone(),
                    _ => return Err("append() requires a list".to_string()),
                };

                values.push(arguments[1].clone());

                Ok(Value::List(values))
            }

            "remove" => {
                if arguments.len() != 2 {
                    return Err("remove() expects two arguments".to_string());
                }

                match &arguments[0] {
                    Value::List(values) => {
                        let index = self.list_index(&arguments[1], values.len())?;

                        let mut values = values.clone();
                        values.remove(index);

                        Ok(Value::List(values))
                    }

                    _ => Err("remove() requires a list".to_string()),
                }
            }

            "keys" => {
                if arguments.len() != 1 {
                    return Err("keys() expects one argument".to_string());
                }

                match &arguments[0] {
                    Value::Dictionary(values) => {
                        let keys = values.keys().map(|key| {
                            match key {
                                DictionaryKey::String(value) =>
                                    Value::String(value.clone()),

                                DictionaryKey::Number(value) =>
                                    Value::Number(*value as f64),

                                DictionaryKey::Boolean(value) =>
                                    Value::Boolean(*value),

                                DictionaryKey::Null => Value::Null,
                            }
                        }).collect();

                        Ok(Value::List(keys))
                    }

                    _ => Err("keys() requires a dictionary".to_string()),
                }
            }

            _ => Err(format!("Unknown built-in function: {}", name)),
        }
    }

    fn import_file(&mut self, path: &str) -> Result<(), String> {
        let resolved = self.base_dir.join(path);

        let canonical = fs::canonicalize(&resolved)
            .map_err(|error| format!("Could not import '{}': {}", path, error))?;

        if self.import_stack.contains(&canonical) {
            return Err(format!("Circular import detected: {}", path));
        }

        if self.imported_files.contains(&canonical) {
            return Ok(());
        }

        let source = fs::read_to_string(&canonical)
            .map_err(|error| format!("Could not read import '{}': {}", path, error))?;

        self.import_stack.insert(canonical.clone());

        let result = self.execute_import(&source, &canonical);

        self.import_stack.remove(&canonical);

        result?;

        self.imported_files.insert(canonical);
        Ok(())
    }

    fn execute_import(
        &mut self,
        source: &str,
        path: &Path,
    ) -> Result<(), String> {
        let mut lexer = crate::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();

        let mut parser = crate::parser::Parser::new(tokens);
        let statements = parser.parse()
            .map_err(|error| format!("Import parser error: {}", error))?;

        let previous_dir = self.base_dir.clone();

        self.base_dir = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        let result = self.execute_block(&statements);

        self.base_dir = previous_dir;

        match result? {
            Flow::Normal => Ok(()),
            Flow::Return(_) => Err("return outside function in imported file".to_string()),
            Flow::Break => Err("break outside loop in imported file".to_string()),
            Flow::Continue => Err("continue outside loop in imported file".to_string()),
        }
    }
}
