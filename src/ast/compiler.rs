use crate::ast::{ASTNode, FunctionArgValue, LogicalOperator, Operator, Parser};
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::sync::Arc;

/// Enum representing different possible values in the IR.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Identifier(String),
    Array(Vec<f64>),
    Map(HashMap<String, Value>),
}

impl Add for Value {
    type Output = Result<Value, String>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            _ => Err("Invalid addition operands".to_string()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, String>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err("Invalid subtraction operands".to_string()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, String>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err("Invalid multiplication operands".to_string()),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, String>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => {
                if b == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            _ => Err("Invalid division operands".to_string()),
        }
    }
}

impl Rem for Value {
    type Output = Result<Value, String>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a % b)),
            _ => Err("Invalid modulo operands".to_string()),
        }
    }
}

/// Enum representing instructions in the IR.
#[derive(Debug, Clone)]
pub enum Instruction {
    Push(Value),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Compare { op: ComparisonOp },
    Logical { op: LogicalOp },
    Not,
    CallFunction { name: String, args: usize },
    PropertyAccess { property: String },
}

#[derive(Debug, Clone)]
pub enum ComparisonOp {
    GreaterThan,
    GreaterEqual,
    LessThan,
    LessEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone)]
pub enum LogicalOp {
    And,
    Or,
}

impl From<LogicalOperator> for LogicalOp {
    fn from(value: LogicalOperator) -> Self {
        match value {
            LogicalOperator::And => LogicalOp::And,
            LogicalOperator::Or => LogicalOp::Or,
        }
    }
}

impl From<LogicalOperator> for Instruction {
    fn from(value: LogicalOperator) -> Self {
        match value {
            LogicalOperator::And => Instruction::Logical { op: LogicalOp::And },
            LogicalOperator::Or => Instruction::Logical { op: LogicalOp::Or },
        }
    }
}

impl From<Operator> for Instruction {
    fn from(value: Operator) -> Self {
        match value {
            Operator::Add => Instruction::Add,
            Operator::Subtract => Instruction::Sub,
            Operator::Multiply => Instruction::Mul,
            Operator::Divide => Instruction::Div,
            Operator::Modulo => Instruction::Mod,
            Operator::GreaterThan => Instruction::Compare {
                op: ComparisonOp::GreaterThan,
            },
            Operator::GreaterThanOrEqual => Instruction::Compare {
                op: ComparisonOp::GreaterEqual,
            },
            Operator::LessThan => Instruction::Compare {
                op: ComparisonOp::LessThan,
            },
            Operator::LessThanOrEqual => Instruction::Compare {
                op: ComparisonOp::LessEqual,
            },
            Operator::Equal => Instruction::Compare {
                op: ComparisonOp::Equal,
            },
            Operator::NotEqual => Instruction::Compare {
                op: ComparisonOp::NotEqual,
            },
        }
    }
}

pub type Function = Arc<dyn Fn(&[Value]) -> Result<Value, String> + Send + Sync>;

pub struct Executor {
    functions: HashMap<String, Function>,
    stack: Vec<Value>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn register_function<F>(&mut self, name: &str, function: F)
    where
        F: Fn(&[Value]) -> Result<Value, String> + Send + Sync + 'static,
    {
        self.functions.insert(name.to_string(), Arc::new(function));
    }

    /// Parse an expression string into an AST.
    pub fn parse_expression(&self, expression: &str) -> Result<ASTNode, String> {
        let ast = Parser::parse_expression(expression)?; // Parse the expression using the grammar.
        Ok(ast)
    }

    /// Evaluates a given expression string against a provided context.
    ///
    /// # Arguments
    ///
    /// * `expression` - A string slice that holds the expression to be evaluated.
    /// * `context` - A reference to a `HashMap` that contains variable values for the expression.
    ///
    /// # Returns
    ///
    /// * `Ok(f64)` if the evaluation succeeds.
    /// * `Err(String)` if parsing or evaluation fails.
    pub fn execute_expression(
        &mut self,
        expression: &str,
        context: &HashMap<String, Value>,
    ) -> Result<Value, String> {
        let ast = self.parse_expression(expression)?;
        self.execute_ast(&ast, &context)
    }

    /// Evaluate a single AST node against a single context.
    pub fn execute_ast(
        &mut self,
        ast: &ASTNode,
        context: &HashMap<String, Value>,
    ) -> Result<Value, String> {
        let instructions = Compiler::compile(&ast);
        self.execute(&instructions, context) // Evaluate the resolved AST.
    }

    pub fn execute(
        &mut self,
        instructions: &[Instruction],
        context: &HashMap<String, Value>, // Context for identifier lookups
    ) -> Result<Value, String> {
        for instr in instructions {
            match instr {
                Instruction::Push(Value::Identifier(id)) => {
                    if let Some(value) = context.get(id) {
                        self.stack.push(value.clone());
                    } else {
                        return Err(format!("Identifier '{}' not found in context", id));
                    }
                }
                Instruction::Push(value) => self.stack.push(value.clone()),

                Instruction::Add
                | Instruction::Sub
                | Instruction::Mul
                | Instruction::Div
                | Instruction::Mod => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    let result = match instr {
                        Instruction::Add => left + right,
                        Instruction::Sub => left - right,
                        Instruction::Mul => left * right,
                        Instruction::Div => left / right,
                        Instruction::Mod => left % right,
                        _ => unreachable!(),
                    };
                    self.stack.push(result?);
                }

                Instruction::Compare { op } => {
                    let right = self.pop_number()?;
                    let left = self.pop_number()?;
                    let result = match op {
                        ComparisonOp::GreaterThan => left > right,
                        ComparisonOp::GreaterEqual => left >= right,
                        ComparisonOp::LessThan => left < right,
                        ComparisonOp::LessEqual => left <= right,
                        ComparisonOp::Equal => left == right,
                        ComparisonOp::NotEqual => left != right,
                    };
                    self.stack.push(Value::Boolean(result));
                }

                Instruction::Logical { op } => {
                    let right = self.pop_boolean()?;
                    let left = self.pop_boolean()?;
                    let result = match op {
                        LogicalOp::And => {
                            if !left {
                                Value::Boolean(false)
                            } else {
                                Value::Boolean(right)
                            }
                        }
                        LogicalOp::Or => {
                            if left {
                                Value::Boolean(true)
                            } else {
                                Value::Boolean(right)
                            }
                        }
                    };
                    self.stack.push(result);
                }

                Instruction::Not => {
                    let value = self.pop_boolean()?;
                    self.stack.push(Value::Boolean(!value));
                }

                Instruction::CallFunction { name, args } => {
                    let mut arguments = Vec::new();
                    for _ in 0..*args {
                        arguments.push(self.stack.pop().ok_or("Stack underflow in function call")?);
                    }
                    arguments.reverse();
                    let function = self
                        .functions
                        .get(name)
                        .ok_or_else(|| format!("Function '{}' not found", name))?;
                    let result = function(&arguments)?;
                    self.stack.push(result);
                }

                Instruction::PropertyAccess { property } => {
                    let base = self
                        .stack
                        .pop()
                        .ok_or("Stack underflow in property access")?;
                    if let Value::Map(map) = base {
                        let value = map
                            .get(property)
                            .ok_or_else(|| format!("Property '{}' not found", property))?;
                        self.stack.push(value.clone());
                    } else {
                        return Err("Property access can only be performed on maps".to_string());
                    }
                }
            }
        }

        // Ensure we return the final value on the stack
        self.stack
            .pop()
            .ok_or("Execution finished with empty stack".to_string())
    }

    fn pop_value(&mut self) -> Result<Value, String> {
        self.stack
            .pop()
            .ok_or("Expected a value on the stack".to_string())
    }

    fn pop_number(&mut self) -> Result<f64, String> {
        match self.stack.pop() {
            Some(Value::Number(n)) => Ok(n),
            Some(Value::Identifier(id)) => {
                Err(format!("Identifier '{}' found where number expected", id))
            }
            _ => Err("Expected a number on the stack".to_string()),
        }
    }

    fn pop_boolean(&mut self) -> Result<bool, String> {
        match self.stack.pop() {
            Some(Value::Boolean(b)) => Ok(b),
            Some(Value::Identifier(id)) => {
                Err(format!("Identifier '{}' found where boolean expected", id))
            }
            _ => Err("Expected a boolean on the stack".to_string()),
        }
    }
}

pub struct Compiler;

impl Compiler {
    /// Parse an expression string into an AST.
    pub fn parse_expression(expression: &str) -> Result<ASTNode, String> {
        let ast = Parser::parse_expression(expression)?; // Parse the expression using the grammar.
        Ok(ast)
    }

    pub fn compile(ast: &ASTNode) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        Self::compile_node(ast, &mut instructions);
        instructions
    }

    pub fn compile_expression(expression: &str) -> Result<Vec<Instruction>, String> {
        let ast = Self::parse_expression(expression)?;
        Self::compile_ast(&ast)
    }

    /// Evaluate a single AST node against a single context.
    pub fn compile_ast(ast: &ASTNode) -> Result<Vec<Instruction>, String> {
        Ok(Compiler::compile(&ast))
    }

    fn compile_node(node: &ASTNode, instructions: &mut Vec<Instruction>) {
        match node {
            ASTNode::Number(n) => instructions.push(Instruction::Push(Value::Number(*n))),
            ASTNode::Boolean(b) => instructions.push(Instruction::Push(Value::Boolean(*b))),
            ASTNode::Identifier(name) => {
                instructions.push(Instruction::Push(Value::Identifier(name.clone())))
            }
            ASTNode::BinaryOperation {
                left,
                operator,
                right,
            } => {
                Self::compile_node(left, instructions);
                Self::compile_node(right, instructions);
                instructions.push(Instruction::from(*operator));
            }
            ASTNode::LogicalOperation {
                left,
                operator,
                right,
            } => {
                Self::compile_node(left, instructions);
                Self::compile_node(right, instructions);
                instructions.push(Instruction::from(*operator));
            }
            ASTNode::NotOperation(inner) => {
                Self::compile_node(inner, instructions);
                instructions.push(Instruction::Not);
            }
            ASTNode::FunctionCall { name, args } => {
                let mut arg_count = 0;
                for (_arg_name, arg_value) in args.args.iter() {
                    match arg_value {
                        FunctionArgValue::Number(n) => {
                            instructions.push(Instruction::Push(Value::Number(*n)))
                        }
                        FunctionArgValue::Boolean(b) => {
                            instructions.push(Instruction::Push(Value::Boolean(*b)))
                        }
                        FunctionArgValue::Identifier(id) => {
                            instructions.push(Instruction::Push(Value::Identifier(id.clone())))
                        }
                        FunctionArgValue::Array(arr) => {
                            instructions.push(Instruction::Push(Value::Array(arr.clone())));
                        }
                    }
                    arg_count += 1;
                }
                instructions.push(Instruction::CallFunction {
                    name: name.clone(),
                    args: arg_count,
                });
            }
            ASTNode::PropertyAccess { base, property } => {
                Self::compile_node(base, instructions);
                instructions.push(Instruction::PropertyAccess {
                    property: property.clone(),
                });
            }
            ASTNode::Group(inner) => Self::compile_node(inner, instructions),
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_execute_simple_arithmetic_expression() {
        let mut executor = Executor::new();
        let result = executor
            .execute_expression("3 + 5", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Number(8.0));
    }

    #[test]
    fn test_execute_simple_logical_expression() {
        let mut executor = Executor::new();
        let result = executor
            .execute_expression("true && false", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_execute_comparison_expression() {
        let mut executor = Executor::new();
        let result = executor
            .execute_expression("10 > 5", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_execute_complex_arithmetic_expression() {
        let mut executor = Executor::new();
        let result = executor
            .execute_expression("2 + 3 * 4 - 5 / 5", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Number(13.0)); // 2 + (3 * 4) - (5 / 5) = 13
    }

    #[test]
    fn test_execute_complex_logical_expression() {
        let mut executor = Executor::new();
        let result = executor
            .execute_expression("true || false && false", &HashMap::new())
            .unwrap();
        assert_eq!(true, true || false && false);
        assert_eq!(result, Value::Boolean(true)); // true || (false && false) = true
    }

    #[test]
    fn test_execute_nested_grouped_expression() {
        let mut executor = Executor::new();
        let result = executor
            .execute_expression("((3 + 2) * (4 - 1)) / 5", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Number(3.0)); // ((3+2) * (4-1)) / 5 = 3
    }

    #[test]
    fn test_execute_with_context() {
        let mut executor = Executor::new();
        let mut context = HashMap::new();
        context.insert("x".to_string(), Value::Number(10.0));
        context.insert("y".to_string(), Value::Number(2.0));

        let result = executor.execute_expression("x * y + 5", &context).unwrap();
        assert_eq!(result, Value::Number(25.0)); // (10 * 2) + 5 = 25
    }

    #[test]
    fn test_execute_function_call() {
        let mut executor = Executor::new();
        executor.register_function("square", |args| {
            if let [Value::Number(n)] = args {
                Ok(Value::Number(n * n))
            } else {
                Err("Expected a single number".to_string())
            }
        });

        let result = executor
            .execute_expression("square(4)", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Number(16.0));
    }

    #[test]
    fn test_execute_nested_function_calls() {
        let mut executor = Executor::new();
        executor.register_function("double", |args| {
            if let [Value::Number(n)] = args {
                Ok(Value::Number(n * 2.0))
            } else {
                Err("Expected a single number".to_string())
            }
        });

        executor.register_function("add", |args| {
            if let [Value::Number(a), Value::Number(b)] = args {
                Ok(Value::Number(a + b))
            } else {
                Err("Expected two numbers".to_string())
            }
        });

        let result = executor
            .execute_expression("add(double(3), double(4))", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Number(14.0)); // add(6, 8) = 14
    }

    #[test]
    fn test_execute_property_access() {
        let mut executor = Executor::new();
        executor.register_function("get_data", |_| {
            let mut map = HashMap::new();
            map.insert("value".to_string(), Value::Number(42.0));
            Ok(Value::Map(map))
        });

        let result = executor
            .execute_expression("get_data().value", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_execute_large_expression() {
        let mut executor = Executor::new();
        let expr = "((5 + 3) * (10 / 2)) + ((4 - 2) * (6 / 3)) - (8 % 3)";
        let result = executor.execute_expression(expr, &HashMap::new()).unwrap();
        assert_eq!(result, Value::Number(42.0)); // ((8 * 5) + (2 * 2)) - 2 = 42
    }

    #[test]
    fn test_execute_division_by_zero() {
        let mut executor = Executor::new();
        let result = executor.execute_expression("10 / 0", &HashMap::new());
        assert_eq!(result, Err("Division by zero".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_undefined_variable() {
        let mut executor = Executor::new();
        let result = executor.execute_expression("x + 2", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_invalid_function_call() {
        let mut executor = Executor::new();
        let result = executor.execute_expression("undefined_function(3)", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_unary_minus() {
        let mut executor = Executor::new();
        let result = executor
            .execute_expression("-5 + 3", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Number(-2.0));
    }

    #[test]
    fn test_execute_not_operator() {
        let mut executor = Executor::new();
        let result = executor
            .execute_expression("!true", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_execute_not_operator_complex() {
        let mut executor = Executor::new();
        let result = executor
            .execute_expression("!(false || true)", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Boolean(false)); // !(false || true) = false
    }

    #[test]
    fn test_execute_mixed_boolean_arithmetic() {
        let mut executor = Executor::new();
        let result = executor
            .execute_expression("(3 > 2) && (5 < 10)", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_execute_complex_mixed_operators() {
        let mut executor = Executor::new();
        let result = executor
            .execute_expression("((4 * 2) > 5) && ((3 + 2) == 5)", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Boolean(true)); // ((8 > 5) && (5 == 5)) = true
    }

    #[test]
    fn test_execute_nested_property_access() {
        let mut executor = Executor::new();
        executor.register_function("get_object", |_| {
            let mut map = HashMap::new();
            let mut inner_map = HashMap::new();
            inner_map.insert("nested".to_string(), Value::Number(99.0));
            map.insert("data".to_string(), Value::Map(inner_map));
            Ok(Value::Map(map))
        });

        let result = executor
            .execute_expression("get_object().data.nested", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Number(99.0));
    }

    #[test]
    fn test_execute_function_call_with_variables() {
        let mut executor = Executor::new();
        executor.register_function("multiply", |args| {
            if let [Value::Number(a), Value::Number(b)] = args {
                Ok(Value::Number(a * b))
            } else {
                Err("Expected two numbers".to_string())
            }
        });

        let mut context = HashMap::new();
        context.insert("x".to_string(), Value::Number(4.0));
        context.insert("y".to_string(), Value::Number(5.0));

        let result = executor
            .execute_expression("multiply(x, y)", &context)
            .unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_execute_function_returning_boolean() {
        let mut executor = Executor::new();
        executor.register_function("is_positive", |args| {
            if let [Value::Number(n)] = args {
                Ok(Value::Boolean(*n > 0.0))
            } else {
                Err("Expected a single number".to_string())
            }
        });

        let result = executor
            .execute_expression("is_positive(-3)", &HashMap::new())
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_execute_large_nested_expression() {
        let mut executor = Executor::new();
        let expr = "((10 * (5 + 3)) / 4) - (2 * ((6 / 3) + (7 - 5)))";
        let result = executor.execute_expression(expr, &HashMap::new()).unwrap();
        assert_eq!(result, Value::Number(12.0)); // ((10 * 8) / 4) - (2 * (2 + 2)) = 12
    }

    #[test]
    fn test_execute_variable_shadows_function() {
        let mut executor = Executor::new();
        executor.register_function("double", |args| {
            if let [Value::Number(n)] = args {
                Ok(Value::Number(n * 2.0))
            } else {
                Err("Expected a single number".to_string())
            }
        });

        let mut context = HashMap::new();
        context.insert("double".to_string(), Value::Number(10.0));

        let result = executor.execute_expression("double * 2", &context).unwrap();
        assert_eq!(result, Value::Number(20.0)); // double is treated as a variable, not a function
    }

    #[test]
    fn test_execute_function_call_with_wrong_args() {
        let mut executor = Executor::new();
        executor.register_function("square", |args| {
            if let [Value::Number(n)] = args {
                Ok(Value::Number(n * n))
            } else {
                Err("Expected a single number".to_string())
            }
        });

        let result = executor.execute_expression("square(3, 4)", &HashMap::new());
        assert!(result.is_err()); // Too many arguments
    }

    #[test]
    fn test_execute_undefined_function() {
        let mut executor = Executor::new();
        let result = executor.execute_expression("unknown_func(3)", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_property_access_on_non_object() {
        let mut executor = Executor::new();
        let result = executor.execute_expression("5.value", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_unexpected_token_error() {
        let mut executor = Executor::new();
        let result = executor.execute_expression("3 + * 5", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_unbalanced_parentheses() {
        let mut executor = Executor::new();
        let result = executor.execute_expression("(3 + (4 * 2)", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_mixed_types_error() {
        let mut executor = Executor::new();
        let result = executor.execute_expression("3 + true", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_access_undefined_property() {
        let mut executor = Executor::new();
        executor.register_function("get_data", |_| {
            let mut map = HashMap::new();
            map.insert("value".to_string(), Value::Number(10.0));
            Ok(Value::Map(map))
        });

        let result = executor.execute_expression("get_data().undefined_prop", &HashMap::new());
        assert!(result.is_err());
    }
}
