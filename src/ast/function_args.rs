use std::collections::HashMap;

/// Enum to represent different types of argument values
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionArgValue {
    // A single number
    Number(f64),
    Identifier(String),
    Array(Vec<f64>),
    Boolean(bool),
}

impl FunctionArgValue {
    /// Helper to get a number or return an error
    pub fn as_number(&self) -> Result<f64, String> {
        if let FunctionArgValue::Number(value) = self {
            Ok(*value)
        } else {
            Err("Expected a Number type".to_string())
        }
    }

    /// Helper to get an array or return an error
    pub fn as_array(&self) -> Result<&[f64], String> {
        if let FunctionArgValue::Array(array) = self {
            Ok(array)
        } else {
            Err("Expected an Array type".to_string())
        }
    }

    /// Helper to get a string or return an error
    pub fn as_string(&self) -> Result<&str, String> {
        if let FunctionArgValue::Identifier(value) = self {
            Ok(value)
        } else {
            Err("Expected a String type".to_string())
        }
    }

    /// Helper to get a boolean or return an error
    pub fn as_boolean(&self) -> Result<bool, String> {
        if let FunctionArgValue::Boolean(value) = self {
            Ok(*value)
        } else {
            Err("Expected a Boolean type".to_string())
        }
    }
}

/// Struct to represent arguments passed to functions
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArgs {
    pub(crate) args: HashMap<String, FunctionArgValue>,
}

impl FunctionArgs {
    /// Creates a new empty FunctionArgs instance
    pub fn new() -> Self {
        Self {
            args: HashMap::new(),
        }
    }

    pub fn with_args(args: HashMap<String, FunctionArgValue>) -> Self {
        Self { args }
    }

    /// Inserts a key-value pair into the arguments
    pub fn insert<T: Into<FunctionArgValue>>(&mut self, key: &str, value: T) {
        self.args.insert(key.to_string(), value.into());
    }

    /// Retrieves an argument by key and expects it to be a number
    pub fn get_number(&self, key: &str) -> Result<f64, String> {
        self.args
            .get(key)
            .ok_or_else(|| format!("Missing argument: {}", key))?
            .as_number()
    }

    /// Retrieves an argument by key and expects it to be an array
    pub fn get_array(&self, key: &str) -> Result<&[f64], String> {
        self.args
            .get(key)
            .ok_or_else(|| format!("Missing argument: {}", key))?
            .as_array()
    }

    /// Retrieves an argument by key and expects it to be a string
    pub fn get_string(&self, key: &str) -> Result<&str, String> {
        self.args
            .get(key)
            .ok_or_else(|| format!("Missing argument: {}", key))?
            .as_string()
    }

    /// Retrieves an argument by key and expects it to be a boolean
    pub fn get_boolean(&self, key: &str) -> Result<bool, String> {
        self.args
            .get(key)
            .ok_or_else(|| format!("Missing argument: {}", key))?
            .as_boolean()
    }

    /// Checks if an argument exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.args.contains_key(key)
    }
}

/// Implement conversion traits for convenience
impl From<f64> for FunctionArgValue {
    fn from(value: f64) -> Self {
        FunctionArgValue::Number(value)
    }
}

impl From<Vec<f64>> for FunctionArgValue {
    fn from(value: Vec<f64>) -> Self {
        FunctionArgValue::Array(value)
    }
}

impl From<String> for FunctionArgValue {
    fn from(value: String) -> Self {
        FunctionArgValue::Identifier(value)
    }
}

impl From<&str> for FunctionArgValue {
    fn from(value: &str) -> Self {
        FunctionArgValue::Identifier(value.to_string())
    }
}

impl From<bool> for FunctionArgValue {
    fn from(value: bool) -> Self {
        FunctionArgValue::Boolean(value)
    }
}
