use std::collections::HashMap;

mod compiler;
mod evaluator;
mod function_args;
mod function_result;
mod parser;

pub use compiler::*;
// pub use evaluator::*;
pub use function_args::*;
pub use function_result::*;
pub use parser::LogicParser as Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    Number(f64),
    Boolean(bool),
    Identifier(String),
    BinaryOperation {
        left: Box<ASTNode>,
        operator: Operator,
        right: Box<ASTNode>,
    },
    LogicalOperation {
        left: Box<ASTNode>,
        operator: LogicalOperator,
        right: Box<ASTNode>,
    },
    NotOperation(Box<ASTNode>),
    Group(Box<ASTNode>),
    FunctionCall {
        name: String,
        args: FunctionArgs,
    },
    PropertyAccess {
        base: Box<ASTNode>,
        property: String,
    },
}

impl ASTNode {
    /// Recursively resolves all identifiers in the AST and replaces them with their values from the context.
    pub fn resolve_identifiers(&self, context: &HashMap<String, f64>) -> Result<ASTNode, String> {
        match self {
            ASTNode::LogicalOperation {
                left,
                operator,
                right,
            } => Ok(ASTNode::LogicalOperation {
                left: Box::new(left.resolve_identifiers(context)?),
                operator: *operator,
                right: Box::new(right.resolve_identifiers(context)?),
            }),
            ASTNode::NotOperation(expression) => Ok(ASTNode::NotOperation(Box::new(
                expression.resolve_identifiers(context)?,
            ))),
            ASTNode::BinaryOperation {
                left,
                operator,
                right,
            } => Ok(ASTNode::BinaryOperation {
                left: Box::new(left.resolve_identifiers(context)?),
                operator: *operator,
                right: Box::new(right.resolve_identifiers(context)?),
            }),
            ASTNode::Group(inner) => {
                let resolved_inner = inner.resolve_identifiers(context)?;
                Ok(ASTNode::Group(Box::new(resolved_inner)))
            }
            ASTNode::FunctionCall { name, args } => {
                let resolved_args = FunctionArgs {
                    args: args
                        .args
                        .iter()
                        .map(|(key, value)| {
                            let resolved_value = match value {
                                FunctionArgValue::Number(num) => Ok(FunctionArgValue::Number(*num)),
                                FunctionArgValue::Boolean(value) => {
                                    Ok(FunctionArgValue::Boolean(*value))
                                }
                                FunctionArgValue::Identifier(identifier) => {
                                    Ok(FunctionArgValue::Identifier(identifier.clone()))
                                }
                                _ => Err("Unsupported argument type".to_string()),
                            }?;
                            Ok((key.clone(), resolved_value))
                        })
                        .collect::<Result<HashMap<String, FunctionArgValue>, String>>()?,
                };
                Ok(ASTNode::FunctionCall {
                    name: name.clone(),
                    args: resolved_args,
                })
            }
            ASTNode::PropertyAccess { base, property } => {
                let resolved_base = base.resolve_identifiers(context)?;
                Ok(ASTNode::PropertyAccess {
                    base: Box::new(resolved_base),
                    property: property.clone(),
                })
            }
            ASTNode::Identifier(ident) => context.get(ident).map_or_else(
                || Err(format!("Identifier '{}' not found in context", ident)),
                |value| Ok(ASTNode::Number(*value)),
            ),
            ASTNode::Number(value) => Ok(ASTNode::Number(value.clone())),
            ASTNode::Boolean(value) => Ok(ASTNode::Boolean(value.clone())),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LogicalOperator {
    And,
    Or,
}

impl LogicalOperator {
    pub fn apply(&self, left: f64, right: f64) -> Result<f64, String> {
        match self {
            LogicalOperator::And => Ok(if left != 0.0 && right != 0.0 {
                1.0
            } else {
                0.0
            }),
            LogicalOperator::Or => Ok(if left != 0.0 || right != 0.0 {
                1.0
            } else {
                0.0
            }),
        }
    }
}

impl TryFrom<&str> for LogicalOperator {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "AND" | "and" | "&&" => Ok(LogicalOperator::And),
            "OR" | "or" | "||" => Ok(LogicalOperator::Or),
            _ => Err(format!("Unknown logical operator: {}", value)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

impl Operator {
    pub fn apply(&self, left: f64, right: f64) -> Result<f64, String> {
        match self {
            Operator::Add => Ok(left + right),
            Operator::Subtract => Ok(left - right),
            Operator::Multiply => Ok(left * right),
            Operator::Divide => {
                if right == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(left / right)
                }
            }
            Operator::Modulo => {
                if right == 0.0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(left % right)
                }
            }
            Operator::GreaterThan => Ok(if left > right { 1.0 } else { 0.0 }),
            Operator::LessThan => Ok(if left < right { 1.0 } else { 0.0 }),
            Operator::GreaterThanOrEqual => Ok(if left >= right { 1.0 } else { 0.0 }),
            Operator::LessThanOrEqual => Ok(if left <= right { 1.0 } else { 0.0 }),
            Operator::Equal => Ok(if left == right { 1.0 } else { 0.0 }),
            Operator::NotEqual => Ok(if left != right { 1.0 } else { 0.0 }),
        }
    }
}

impl TryFrom<&str> for Operator {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Operator::Add),
            "-" => Ok(Operator::Subtract),
            "*" => Ok(Operator::Multiply),
            "/" => Ok(Operator::Divide),
            "%" => Ok(Operator::Modulo),
            ">" => Ok(Operator::GreaterThan),
            "<" => Ok(Operator::LessThan),
            ">=" => Ok(Operator::GreaterThanOrEqual),
            "<=" => Ok(Operator::LessThanOrEqual),
            "==" => Ok(Operator::Equal),
            "!=" => Ok(Operator::NotEqual),
            _ => Err(format!("Unknown operator: {}", value)),
        }
    }
}
