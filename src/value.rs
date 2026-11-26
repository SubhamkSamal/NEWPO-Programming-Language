
use std::collections::HashMap;
use std::fmt;

use crate::ast::Stmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DictionaryKey {
    String(String),
    Number(i64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone)]
pub struct UserFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,

    List(Vec<Value>),

    Dictionary(HashMap<DictionaryKey, Value>),

    Function(UserFunction),

    BuiltinFunction(String),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Null => "null",
            Value::List(_) => "list",
            Value::Dictionary(_) => "dictionary",
            Value::Function(_) => "function",
            Value::BuiltinFunction(_) => "built-in function",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(value) => *value,
            Value::Null => false,
            Value::Number(value) => *value != 0.0,
            Value::String(value) => !value.is_empty(),
            Value::List(value) => !value.is_empty(),
            Value::Dictionary(value) => !value.is_empty(),
            Value::Function(_) => true,
            Value::BuiltinFunction(_) => true,
        }
    }

    pub fn to_key(&self) -> Result<DictionaryKey, String> {
        match self {
            Value::String(value) => {
                Ok(DictionaryKey::String(value.clone()))
            }

            Value::Number(value) => {
                if !value.is_finite()
                    || value.fract() != 0.0
                    || *value < i64::MIN as f64
                    || *value >= i64::MAX as f64
                {
                    return Err(
                        "Dictionary number keys must be whole numbers"
                            .to_string()
                    );
                }

                Ok(DictionaryKey::Number(*value as i64))
            }

            Value::Boolean(value) => {
                Ok(DictionaryKey::Boolean(*value))
            }

            Value::Null => Ok(DictionaryKey::Null),

            _ => Err(format!(
                "{} cannot be used as a dictionary key",
                self.type_name()
            )),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,

            (Value::String(a), Value::String(b)) => a == b,

            (Value::Boolean(a), Value::Boolean(b)) => a == b,

            (Value::Null, Value::Null) => true,

            (Value::List(a), Value::List(b)) => a == b,

            (Value::Dictionary(a), Value::Dictionary(b)) => a == b,

            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(value) => write!(f, "{}", value),

            Value::String(value) => write!(f, "{}", value),

            Value::Boolean(value) => write!(f, "{}", value),

            Value::Null => write!(f, "null"),

            Value::List(values) => {
                write!(f, "[")?;

                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", value)?;
                }

                write!(f, "]")
            }

            Value::Dictionary(values) => {
                write!(f, "{{")?;

                for (index, (key, value)) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    match key {
                        DictionaryKey::String(key) => {
                            write!(f, "\"{}\"", key)?;
                        }

                        DictionaryKey::Number(key) => {
                            write!(f, "{}", key)?;
                        }

                        DictionaryKey::Boolean(key) => {
                            write!(f, "{}", key)?;
                        }

                        DictionaryKey::Null => {
                            write!(f, "null")?;
                        }
                    }

                    write!(f, ": {}", value)?;
                }

                write!(f, "}}")
            }

            Value::Function(function) => {
                write!(f, "<function {}>", function.name)
            }

            Value::BuiltinFunction(name) => {
                write!(f, "<built-in {}>", name)
            }
        }
    }
}
