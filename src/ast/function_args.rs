// use crate::ast::ASTNode;
use std::collections::HashMap;
// use std::hash::{Hash, Hasher};

/// Enum to represent different types of argument values
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionArgValue {
    // A single number
    Number(f64),
    Identifier(String),
    // // An array of numbers
    Array(Vec<f64>),
    // KeyValue(HashMap<String, f64>), // Key-value pairs for complex functions
    // String(String),                 // A string value
    // A boolean value
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

    // /// Helper to get a key-value map or return an error
    // pub fn as_key_value(&self) -> Result<&HashMap<String, f64>, String> {
    //     if let FunctionArgValue::KeyValue(map) = self {
    //         Ok(map)
    //     } else {
    //         Err("Expected a KeyValue type".to_string())
    //     }
    // }

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

    // /// Retrieves an argument by key and expects it to be a key-value map
    // pub fn get_key_value(&self, key: &str) -> Result<&HashMap<String, f64>, String> {
    //     self.args
    //         .get(key)
    //         .ok_or_else(|| format!("Missing argument: {}", key))?
    //         .as_key_value()
    // }

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

// impl FunctionArgs {
//     /// Constructs `FunctionArgs` from a vector of `ASTNode` values.
//     ///
//     /// # Arguments
//     /// - `args`: A vector of AST nodes representing the function's arguments.
//     /// - `context`: A map containing variable values for identifiers in the AST.
//     pub fn from_ast(
//         args: &[ASTNode],
//         evaluator: &crate::Evaluator,
//         context: &HashMap<String, f64>,
//     ) -> Result<Self, String> {
//         let mut function_args = FunctionArgs::new();
//
//         for (index, arg) in args.iter().enumerate() {
//             let value = match arg {
//                 ASTNode::Number(n) => FunctionArgValue::Number(*n),
//
//                 ASTNode::Identifier(ident) => {
//                     // Look up the identifier in the context
//                     if let Some(val) = context.get(ident) {
//                         FunctionArgValue::Number(*val)
//                     } else {
//                         return Err(format!("Identifier '{}' not found in context", ident));
//                     }
//                 }
//
//                 ASTNode::FunctionCall { name, args } => {
//                     // Evaluate the nested function call
//                     let function = evaluator
//                         .functions
//                         .get(name)
//                         .ok_or_else(|| format!("Function '{}' not found", name))?;
//
//                     let nested_args = FunctionArgs::from_ast(args, evaluator, context)?;
//                     let result = function(&nested_args)?;
//                     FunctionArgValue::Number(result)
//                 }
//
//                 _ => return Err(format!("Unsupported argument type in AST: {:?}", arg)),
//             };
//
//             // Insert the value into the function arguments with a numeric key
//             function_args.insert(&index.to_string(), value);
//         }
//
//         Ok(function_args)
//     }
// }

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

// impl From<HashMap<String, f64>> for FunctionArgValue {
//     fn from(value: HashMap<String, f64>) -> Self {
//         FunctionArgValue::KeyValue(value)
//     }
// }
//
// impl From<String> for FunctionArgValue {
//     fn from(value: String) -> Self {
//         FunctionArgValue::String(value)
//     }
// }
//
// impl From<&str> for FunctionArgValue {
//     fn from(value: &str) -> Self {
//         FunctionArgValue::String(value.to_string())
//     }
// }

impl From<bool> for FunctionArgValue {
    fn from(value: bool) -> Self {
        FunctionArgValue::Boolean(value)
    }
}
