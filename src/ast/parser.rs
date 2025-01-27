use crate::ast::{ASTNode, FunctionArgValue, FunctionArgs, LogicalOperator};
use log::debug;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "./expression.pest"] // Link to the grammar file
pub struct LogicParser;

impl LogicParser {
    pub fn parse_expression(input: &str) -> Result<ASTNode, String> {
        debug!("Input: {:#?}", input);
        let parsed = LogicParser::parse(Rule::expression, input)
            .map_err(|e| format!("Parsing error: {}", e))?;
        debug!("Parse: {:#?}", parsed);
        let root = parsed.into_iter().next().ok_or("Empty expression")?;
        Self::build_ast(root)
    }

    fn build_ast(pair: Pair<Rule>) -> Result<ASTNode, String> {
        match pair.as_rule() {
            Rule::logical_expression => Self::build_ast(pair.into_inner().next().unwrap()),
            Rule::or_expression => {
                let mut pairs = pair.into_inner();
                let mut left = Self::build_ast(pairs.next().unwrap())?;
                while let Some(pair) = pairs.next() {
                    let right = Self::build_ast(pair)?;
                    left = ASTNode::LogicalOperation {
                        left: Box::new(left),
                        operator: LogicalOperator::Or,
                        right: Box::new(right),
                    };
                }
                Ok(left)
            }

            Rule::and_expression => {
                let mut pairs = pair.into_inner();
                let mut left = Self::build_ast(pairs.next().unwrap())?;
                while let Some(pair) = pairs.next() {
                    let right = Self::build_ast(pair)?;
                    left = ASTNode::LogicalOperation {
                        left: Box::new(left),
                        operator: LogicalOperator::And,
                        right: Box::new(right),
                    };
                }
                Ok(left)
            }
            Rule::not_expression => {
                let mut pairs = pair.into_inner();
                let first = pairs.next().unwrap();
                if first.as_rule() == Rule::NOT {
                    Ok(ASTNode::NotOperation(Box::new(Self::build_ast(
                        pairs.next().unwrap(),
                    )?)))
                } else {
                    Self::build_ast(first)
                }
            }
            Rule::comparison => Self::build_ast(pair.into_inner().next().unwrap()),

            Rule::binary_operation => {
                let mut pairs = pair.into_inner();
                let left = Self::build_ast(pairs.next().ok_or("Expected left operand")?)?;
                let operator = pairs
                    .next()
                    .ok_or("Expected operator")?
                    .as_str()
                    .try_into()?;
                let right = Self::build_ast(pairs.next().ok_or("Expected right operand")?)?;
                Ok(ASTNode::BinaryOperation {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                })
            }

            Rule::function_call => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let args = parse_function_args(inner.next());
                Ok(ASTNode::FunctionCall { name, args })
            }
            Rule::property_access => {
                let mut pairs = pair.into_inner();
                let mut base = Self::build_ast(pairs.next().unwrap())?;
                while let Some(property) = pairs.next() {
                    let property = property.as_str().to_string();
                    base = ASTNode::PropertyAccess {
                        base: Box::new(base),
                        property,
                    };
                }
                Ok(base)
            }
            Rule::group => Self::build_ast(pair.into_inner().next().unwrap()),

            Rule::identifier => Ok(ASTNode::Identifier(pair.as_str().to_string())),
            Rule::number => pair
                .as_str()
                .parse::<f64>()
                .map(ASTNode::Number)
                .map_err(|_| "Invalid number".to_string()),
            _ => Err(format!("Unexpected rule: {:?}", pair.as_rule())),
        }
    }
}

fn parse_function_args(pair: Option<pest::iterators::Pair<Rule>>) -> FunctionArgs {
    let mut args = HashMap::new();
    if let Some(inner) = pair {
        for named_arg in inner.into_inner() {
            let mut inner = named_arg.into_inner();
            let key = inner.next().unwrap().as_str().to_string();
            let value = parse_value(inner.next().unwrap());
            args.insert(key, value);
        }
    }
    FunctionArgs { args }
}

fn parse_value(pair: pest::iterators::Pair<Rule>) -> FunctionArgValue {
    match pair.as_rule() {
        Rule::number => FunctionArgValue::Number(pair.as_str().parse().unwrap()),
        Rule::identifier => FunctionArgValue::Identifier(pair.as_str().to_string()),
        // Rule::group => {
        //     let inner = pair.into_inner();
        //     let node = LogicParser::build_ast(inner).unwrap(); // Panic for invalid group
        //     FunctionArgValue::Expression(Box::new(node))
        // }
        _ => panic!("Unexpected value type: {:?}", pair),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ASTNode, Operator};
    use std::collections::HashMap;

    #[test]
    fn test_simple_binary_expression() {
        let input = "price > 100";
        let ast = LogicParser::parse_expression(input).unwrap();
        let expected_ast = ASTNode::BinaryOperation {
            left: Box::new(ASTNode::Identifier("price".to_string())),
            operator: Operator::GreaterThan,
            right: Box::new(ASTNode::Number(100.0)),
        };
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn test_and_expression() {
        let input = "price > 100 AND volume < 5000";
        let ast = LogicParser::parse_expression(input).unwrap();
        let expected_ast = ASTNode::LogicalOperation {
            left: Box::new(ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("price".to_string())),
                operator: Operator::GreaterThan,
                right: Box::new(ASTNode::Number(100.0)),
            }),
            operator: LogicalOperator::And,
            right: Box::new(ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("volume".to_string())),
                operator: Operator::LessThan,
                right: Box::new(ASTNode::Number(5000.0)),
            }),
        };
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn test_or_expression() {
        let input = "price > 100 OR volume < 5000";
        let ast = LogicParser::parse_expression(input).unwrap();
        let expected_ast = ASTNode::LogicalOperation {
            left: Box::new(ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("price".to_string())),
                operator: Operator::GreaterThan,
                right: Box::new(ASTNode::Number(100.0)),
            }),
            operator: LogicalOperator::Or,
            right: Box::new(ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("volume".to_string())),
                operator: Operator::LessThan,
                right: Box::new(ASTNode::Number(5000.0)),
            }),
        };
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn test_nested_logical_expression() {
        let input = "(price > 100 AND volume < 5000) OR volume >= 3000";
        let ast = LogicParser::parse_expression(input).unwrap();
        let expected_ast = ASTNode::LogicalOperation {
            left: Box::new(ASTNode::LogicalOperation {
                left: Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier("price".to_string())),
                    operator: Operator::GreaterThan,
                    right: Box::new(ASTNode::Number(100.0)),
                }),
                operator: LogicalOperator::And,
                right: Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier("volume".to_string())),
                    operator: Operator::LessThan,
                    right: Box::new(ASTNode::Number(5000.0)),
                }),
            }),
            operator: LogicalOperator::Or,
            right: Box::new(ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("volume".to_string())),
                operator: Operator::GreaterThanOrEqual,
                right: Box::new(ASTNode::Number(3000.0)),
            }),
        };
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn test_function_call() {
        let input = "ema(price: close, period: 10)";
        let ast = LogicParser::parse_expression(input).unwrap();
        let mut args = HashMap::new();
        args.insert(
            "price".to_string(),
            FunctionArgValue::Identifier("close".to_string()),
        );
        args.insert("period".to_string(), FunctionArgValue::Number(10.0));

        let expected_ast = ASTNode::FunctionCall {
            name: "ema".to_string(),
            args: FunctionArgs { args },
        };
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn test_property_access() {
        let input = "indicator.ema";
        let ast = LogicParser::parse_expression(input).unwrap();
        let expected_ast = ASTNode::PropertyAccess {
            base: Box::new(ASTNode::Identifier("indicator".to_string())),
            property: "ema".to_string(),
        };
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn test_grouped_expression() {
        let input = "((price > 100) AND (volume < 5000)) OR (volume >= 3000)";
        let ast = LogicParser::parse_expression(input).unwrap();
        let expected_ast = ASTNode::LogicalOperation {
            left: Box::new(ASTNode::LogicalOperation {
                left: Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier("price".to_string())),
                    operator: Operator::GreaterThan,
                    right: Box::new(ASTNode::Number(100.0)),
                }),
                operator: LogicalOperator::And,
                right: Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier("volume".to_string())),
                    operator: Operator::LessThan,
                    right: Box::new(ASTNode::Number(5000.0)),
                }),
            }),
            operator: LogicalOperator::Or,
            right: Box::new(ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("volume".to_string())),
                operator: Operator::GreaterThanOrEqual,
                right: Box::new(ASTNode::Number(3000.0)),
            }),
        };
        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn test_not_expression() {
        let input = "NOT (price > 100)";
        let ast = LogicParser::parse_expression(input).unwrap();

        let expected = ASTNode::NotOperation(Box::new(ASTNode::BinaryOperation {
            left: Box::new(ASTNode::Identifier("price".to_string())),
            operator: Operator::GreaterThan,
            right: Box::new(ASTNode::Number(100.0)),
        }));

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_chained_and_or_with_not() {
        let input = "price > 100 AND NOT volume < 5000 OR volume >= 3000";
        let ast = LogicParser::parse_expression(input).unwrap();

        let expected = ASTNode::LogicalOperation {
            left: Box::new(ASTNode::LogicalOperation {
                left: Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier("price".to_string())),
                    operator: Operator::GreaterThan,
                    right: Box::new(ASTNode::Number(100.0)),
                }),
                operator: LogicalOperator::And,
                right: Box::new(ASTNode::NotOperation(Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier("volume".to_string())),
                    operator: Operator::LessThan,
                    right: Box::new(ASTNode::Number(5000.0)),
                }))),
            }),
            operator: LogicalOperator::Or,
            right: Box::new(ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("volume".to_string())),
                operator: Operator::GreaterThanOrEqual,
                right: Box::new(ASTNode::Number(3000.0)),
            }),
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_chained_property_access() {
        let input = "indicator.result.signal";
        let ast = LogicParser::parse_expression(input).unwrap();

        let expected = ASTNode::PropertyAccess {
            base: Box::new(ASTNode::PropertyAccess {
                base: Box::new(ASTNode::Identifier("indicator".to_string())),
                property: "result".to_string(),
            }),
            property: "signal".to_string(),
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_function_call_no_args() {
        let input = "random()";
        let ast = LogicParser::parse_expression(input).unwrap();

        let expected = ASTNode::FunctionCall {
            name: "random".to_string(),
            args: FunctionArgs {
                args: HashMap::new(),
            },
        };

        assert_eq!(ast, expected);
    }
    #[test]
    fn test_function_call_positional_and_named_args() {
        let input = "sma(price: close, period: 10)";
        let ast = LogicParser::parse_expression(input).unwrap();

        let expected = ASTNode::FunctionCall {
            name: "sma".to_string(),
            args: FunctionArgs {
                args: HashMap::from([
                    (
                        "price".to_string(),
                        FunctionArgValue::Identifier("close".to_string()),
                    ),
                    ("period".to_string(), FunctionArgValue::Number(10.0)),
                ]),
            },
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_invalid_syntax() {
        let input = "price > AND volume < 5000";
        let result = LogicParser::parse_expression(input);

        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .contains("expected group, identifier, or number"));
    }

    #[test]
    fn test_operator_without_operand() {
        let input = "price >";
        let result = LogicParser::parse_expression(input);

        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .contains("expected group, identifier, or number"));
    }

    #[test]
    fn test_single_identifier() {
        let input = "price";
        let result = LogicParser::parse_expression(input);

        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Parsing error"));
    }

    #[test]
    fn test_chained_logical_operations() {
        let input = "price > 100 AND volume < 5000 AND volume >= 3000";
        let ast = LogicParser::parse_expression(input).unwrap();

        let expected = ASTNode::LogicalOperation {
            left: Box::new(ASTNode::LogicalOperation {
                left: Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier("price".to_string())),
                    operator: Operator::GreaterThan,
                    right: Box::new(ASTNode::Number(100.0)),
                }),
                operator: LogicalOperator::And,
                right: Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier("volume".to_string())),
                    operator: Operator::LessThan,
                    right: Box::new(ASTNode::Number(5000.0)),
                }),
            }),
            operator: LogicalOperator::And,
            right: Box::new(ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("volume".to_string())),
                operator: Operator::GreaterThanOrEqual,
                right: Box::new(ASTNode::Number(3000.0)),
            }),
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_property_access_with_function_call() {
        let input = "ema(price: close, period: 10).signal";
        let ast = LogicParser::parse_expression(input).unwrap();

        let expected = ASTNode::PropertyAccess {
            base: Box::new(ASTNode::FunctionCall {
                name: "ema".to_string(),
                args: FunctionArgs {
                    args: HashMap::from([
                        (
                            "price".to_string(),
                            FunctionArgValue::Identifier("close".to_string()),
                        ),
                        ("period".to_string(), FunctionArgValue::Number(10.0)),
                    ]),
                },
            }),
            property: "signal".to_string(),
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_function_call_with_multiple_args() {
        let input = "ema(price: close, period: 10)";
        let ast = LogicParser::parse_expression(input).unwrap();

        let expected = ASTNode::FunctionCall {
            name: "ema".to_string(),
            args: FunctionArgs {
                args: HashMap::from([
                    (
                        "price".to_string(),
                        FunctionArgValue::Identifier("close".to_string()),
                    ),
                    ("period".to_string(), FunctionArgValue::Number(10.0)),
                ]),
            },
        };

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_missing_operator() {
        let input = "price > 100 volume < 5000";
        let ast = LogicParser::parse_expression(input).unwrap();

        // LogicParser::parse only parses the first part price > 100, and the rest is ignored.
        // Couldn't find a way to make it fail on the second part volume < 5000.
        //
        // TODO: find a way to throw an error on parse.
        assert_eq!(
            ast,
            ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("price".to_string())),
                operator: Operator::GreaterThan,
                right: Box::new(ASTNode::Number(100.0)),
            }
        );
    }

    #[test]
    fn test_unbalanced_parentheses() {
        let input = "(price > 100 AND volume < 5000";
        let result = LogicParser::parse_expression(input);

        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Parsing error"));
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let result = LogicParser::parse_expression(input);

        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Parsing error"));
    }

    #[test]
    fn test_complex_nested_expression() {
        let input = "(price > 100 AND (volume < 5000 OR volume >= 3000)) OR (price < 50 AND volume == 1000)";
        let ast = LogicParser::parse_expression(input).unwrap();

        let expected = ASTNode::LogicalOperation {
            left: Box::new(ASTNode::LogicalOperation {
                left: Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier("price".to_string())),
                    operator: Operator::GreaterThan,
                    right: Box::new(ASTNode::Number(100.0)),
                }),
                operator: LogicalOperator::And,
                right: Box::new(ASTNode::LogicalOperation {
                    left: Box::new(ASTNode::BinaryOperation {
                        left: Box::new(ASTNode::Identifier("volume".to_string())),
                        operator: Operator::LessThan,
                        right: Box::new(ASTNode::Number(5000.0)),
                    }),
                    operator: LogicalOperator::Or,
                    right: Box::new(ASTNode::BinaryOperation {
                        left: Box::new(ASTNode::Identifier("volume".to_string())),
                        operator: Operator::GreaterThanOrEqual,
                        right: Box::new(ASTNode::Number(3000.0)),
                    }),
                }),
            }),
            operator: LogicalOperator::Or,
            right: Box::new(ASTNode::LogicalOperation {
                left: Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier("price".to_string())),
                    operator: Operator::LessThan,
                    right: Box::new(ASTNode::Number(50.0)),
                }),
                operator: LogicalOperator::And,
                right: Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier("volume".to_string())),
                    operator: Operator::Equal,
                    right: Box::new(ASTNode::Number(1000.0)),
                }),
            }),
        };

        assert_eq!(ast, expected);
    }
}
