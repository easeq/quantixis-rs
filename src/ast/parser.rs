use crate::ast::{ASTNode, FunctionArgValue, FunctionArgs, LogicalOperator, Operator};
use log::debug;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "./expression.pest"] // Link to the grammar file
pub struct LogicParser;

impl LogicParser {
    pub fn parse_expression(input: &str) -> Result<ASTNode, String> {
        debug!("Parsing expression: {}", input);
        let parse_result = LogicParser::parse(Rule::expression, input)
            .map_err(|e| format!("Parse error: {}", e))?
            .next()
            .ok_or_else(|| "Failed to parse expression".to_string())?;

        debug!("Parse result: {:#?}", parse_result);
        Self::build_logical_expression(parse_result)
    }
    //
    // fn build_expression(pair: Pair<Rule>) -> Result<ASTNode, String> {
    //     match pair.as_rule() {
    //         Rule::logical_expression | Rule::or_expression | Rule::and_expression => {
    //             Self::build_logical_expression(pair)
    //         }
    //         Rule::comparison_expression => Self::build_comparison_expression(pair),
    //         Rule::arithmetic_expression => Self::build_arithmetic_expression(pair),
    //         Rule::primary_expression => Self::build_primary_expression(pair),
    //         _ => Err(format!("Unexpected rule: {:?}", pair.as_rule())),
    //     }
    // }

    fn build_logical_expression(pair: Pair<Rule>) -> Result<ASTNode, String> {
        let mut pairs = pair.into_inner();
        debug!("Building logical expression: {:#?}", pairs);
        Self::build_or_expression(pairs.next().unwrap())
    }

    fn build_or_expression(pair: Pair<Rule>) -> Result<ASTNode, String> {
        let mut pairs = pair.into_inner();
        debug!("Building OR expression: {:#?}", pairs);
        let mut node = Self::build_and_expression(pairs.next().unwrap())?;

        while let Some(operator_pair) = pairs.next() {
            let operator = match operator_pair.as_rule() {
                Rule::OR => LogicalOperator::Or,
                _ => return Err(format!("Unexpected logical operator: {:?}", operator_pair)),
            };

            let right = Self::build_and_expression(pairs.next().unwrap())?;
            node = ASTNode::LogicalOperation {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn build_and_expression(pair: Pair<Rule>) -> Result<ASTNode, String> {
        let mut pairs = pair.into_inner();
        debug!("Building AND expression: {:?}", pairs);
        let p = pairs.next().unwrap();
        debug!("AND expression: {:#?}", p);
        let mut node = Self::build_not_expression(p)?;
        debug!("Initial AND node: {:#?}", node);

        // debug!("AND next pair: {:#?}", pairs.next().unwrap());

        while let Some(operator_pair) = pairs.next() {
            // debug!("Pairs: {:#?}", pairs);
            debug!("AND operator: {:?}", operator_pair);
            let operator = match operator_pair.as_rule() {
                Rule::AND => LogicalOperator::And,
                _ => return Err(format!("Unexpected logical operator: {:?}", operator_pair)),
            };

            let right = Self::build_not_expression(pairs.next().unwrap())?;
            node = ASTNode::LogicalOperation {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn build_not_expression(pair: Pair<Rule>) -> Result<ASTNode, String> {
        let mut pairs = pair.into_inner();

        debug!("Building not expression: {:?}", pairs);
        let operator_pair = pairs.next().unwrap();
        debug!("NOT operator: {:?}", operator_pair);
        if operator_pair.as_rule() == Rule::NOT {
            let inner_node = Self::build_comparison_expression(pairs.next().unwrap())?;
            return Ok(ASTNode::NotOperation(Box::new(inner_node)));
        } else {
            Self::build_comparison_expression(operator_pair)
        }
    }

    fn build_comparison_expression(pair: Pair<Rule>) -> Result<ASTNode, String> {
        debug!("Building comparison expression: {:?}", pair);
        let mut pairs = pair.into_inner();
        let mut node = Self::build_arithmetic_expression(pairs.next().unwrap())?;
        debug!("Initial comparison node: {:#?}", node);

        while let Some(operator_pair) = pairs.next() {
            let operator = match operator_pair.as_str() {
                ">" => Operator::GreaterThan,
                ">=" => Operator::GreaterThanOrEqual,
                "<" => Operator::LessThan,
                "<=" => Operator::LessThanOrEqual,
                "==" => Operator::Equal,
                "!=" => Operator::NotEqual,
                _ => {
                    return Err(format!(
                        "Unexpected comparison operator: {:?}",
                        operator_pair
                    ))
                }
            };

            let right = Self::build_arithmetic_expression(pairs.next().unwrap())?;
            node = ASTNode::BinaryOperation {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn build_arithmetic_expression(pair: Pair<Rule>) -> Result<ASTNode, String> {
        debug!("Building arithmetic expression: {:?}", pair);
        let mut pairs = pair.into_inner();
        let mut node = Self::build_term(pairs.next().unwrap())?;
        debug!("Initial arithmetic node: {:#?}", node);
        while let Some(operator_pair) = pairs.next() {
            let operator = match operator_pair.as_rule() {
                Rule::PLUS => Operator::Add,
                Rule::MINUS => Operator::Subtract,
                Rule::comparison_operator => operator_pair.as_str().try_into()?,
                _ => {
                    return Err(format!(
                        "Unexpected arithmetic operator: {:?}",
                        operator_pair
                    ))
                }
            };

            let right = Self::build_term(pairs.next().unwrap())?;
            node = ASTNode::BinaryOperation {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn build_term(pair: Pair<Rule>) -> Result<ASTNode, String> {
        debug!("Building term: {:?}", pair);
        let mut pairs = pair.into_inner();
        let mut node = Self::build_factor(pairs.next().unwrap())?;

        while let Some(operator_pair) = pairs.next() {
            let operator = match operator_pair.as_rule() {
                Rule::STAR => Operator::Multiply,
                Rule::SLASH => Operator::Divide,
                Rule::MOD => Operator::Modulo,
                _ => return Err(format!("Unexpected term operator: {:?}", operator_pair)),
            };

            let right = Self::build_factor(pairs.next().unwrap())?;
            node = ASTNode::BinaryOperation {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn build_factor(pair: Pair<Rule>) -> Result<ASTNode, String> {
        let mut pairs = pair.into_inner();
        debug!("Building factor: {:?}", pairs);

        if let Some(operator_pair) = pairs.peek() {
            if operator_pair.as_rule() == Rule::NOT {
                pairs.next(); // Consume the NOT operator
                let inner_node = Self::build_factor(pairs.next().unwrap())?;
                return Ok(ASTNode::NotOperation(Box::new(inner_node)));
            }
        }

        let primary = pairs.next().ok_or("Expected a primary expression")?;
        Self::build_primary_expression(primary)
    }

    fn build_primary_expression(pair: Pair<Rule>) -> Result<ASTNode, String> {
        debug!("Building primary expression: {:?}", pair);
        match pair.as_rule() {
            Rule::number => {
                let value = pair.as_str().parse::<f64>().unwrap();
                Ok(ASTNode::Number(value))
            }
            Rule::identifier => Ok(ASTNode::Identifier(pair.as_str().to_string())),
            Rule::group => {
                let inner = pair.into_inner().next().unwrap();
                Self::build_logical_expression(inner)
            }
            Rule::function_call => Self::build_function_call(pair),
            Rule::property_access => Self::build_property_access(pair),
            _ => {
                debug!("Unexpected rule in primary expression: {:?}", pair);
                Err(format!(
                    "Unexpected rule in primary expression: {:?}",
                    pair.as_rule()
                ))
            }
        }
    }

    fn build_function_call(pair: Pair<Rule>) -> Result<ASTNode, String> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let args = parse_function_args(inner.next());
        Ok(ASTNode::FunctionCall { name, args })
    }

    fn build_property_access(pair: Pair<Rule>) -> Result<ASTNode, String> {
        let mut pairs = pair.into_inner();
        let mut base = Self::build_primary_expression(pairs.next().unwrap())?;
        while let Some(property) = pairs.next() {
            let property = property.as_str().to_string();
            base = ASTNode::PropertyAccess {
                base: Box::new(base),
                property,
            };
        }
        Ok(base)
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
        assert!(result.err().unwrap().contains("Parse error"));
    }

    #[test]
    fn test_operator_without_operand() {
        let input = "price >";
        let result = LogicParser::parse_expression(input);

        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Parse error"));
    }

    #[test]
    fn test_single_identifier() {
        let input = "price";
        let ast = LogicParser::parse_expression(input).unwrap();

        // TODO: Should this be an error?
        assert_eq!(ast, ASTNode::Identifier("price".to_string()));
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
        let result = LogicParser::parse_expression(input);

        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Parse error"));
    }

    #[test]
    fn test_unbalanced_parentheses() {
        let input = "(price > 100 AND volume < 5000";
        let result = LogicParser::parse_expression(input);

        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Parse error"));
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let result = LogicParser::parse_expression(input);

        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Parse error"));
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

    #[test]
    fn test_invalid_binary_operations() {
        let inputs = vec![
            "price > AND volume < 5000",
            "price >",
            "volume <=",
            "price == OR volume != 200",
        ];

        for input in inputs {
            assert!(
                LogicParser::parse_expression(input).is_err(),
                "Input '{}' should fail to parse, but it succeeded",
                input
            );
        }
    }

    #[test]
    fn test_empty_logical_operations() {
        let inputs = vec!["AND", "OR", "NOT", "price > 100 AND OR volume < 5000"];

        for input in inputs {
            assert!(
                LogicParser::parse_expression(input).is_err(),
                "Input '{}' should fail to parse, but it succeeded",
                input
            );
        }
    }

    #[test]
    fn test_complex_nested_logical_operations() {
        let input = "(price > 100 AND (volume < 5000 OR NOT (open <= 300))) OR close > 1000";
        let ast = LogicParser::parse_expression(input).unwrap();

        let expected_ast = ASTNode::LogicalOperation {
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
                    right: Box::new(ASTNode::NotOperation(Box::new(ASTNode::BinaryOperation {
                        left: Box::new(ASTNode::Identifier("open".to_string())),
                        operator: Operator::LessThanOrEqual,
                        right: Box::new(ASTNode::Number(300.0)),
                    }))),
                }),
            }),
            operator: LogicalOperator::Or,
            right: Box::new(ASTNode::BinaryOperation {
                left: Box::new(ASTNode::Identifier("close".to_string())),
                operator: Operator::GreaterThan,
                right: Box::new(ASTNode::Number(1000.0)),
            }),
        };

        assert_eq!(ast, expected_ast);
    }

    #[test]
    fn test_invalid_property_access() {
        let inputs = vec!["price.", "volume..open", "price > 100 AND volume."];

        for input in inputs {
            assert!(
                LogicParser::parse_expression(input).is_err(),
                "Input '{}' should fail to parse, but it succeeded",
                input
            );
        }
    }

    #[test]
    fn test_function_call_with_invalid_syntax() {
        let inputs = vec![
            "ema(price: , period: 10)",     // Missing argument value
            "ema(price, period:)",          // Missing value for named arg
            "ema(price period: 10)",        // Missing comma between args
            "ema(price: close, period 10)", // Missing colon
            "ema(price: close period: 10)", // Missing comma
        ];

        for input in inputs {
            assert!(
                LogicParser::parse_expression(input).is_err(),
                "Input '{}' should fail to parse, but it succeeded",
                input
            );
        }
    }

    #[test]
    fn test_multiple_consecutive_operators() {
        let inputs = vec![
            "price >> 100",
            "price >>> 100",
            "price > OR volume < 5000",
            "price > > volume",
        ];

        for input in inputs {
            assert!(
                LogicParser::parse_expression(input).is_err(),
                "Input '{}' should fail to parse, but it succeeded",
                input
            );
        }
    }

    #[test]
    fn test_excess_whitespace() {
        let input = "price    >     100     AND   volume    <    5000";
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
    fn test_input_with_unsupported_characters() {
        let inputs = vec![
            "price > 100 @ volume < 5000",
            "price > 100 # volume < 5000",
            "price > 100 $",
        ];

        for input in inputs {
            assert!(
                LogicParser::parse_expression(input).is_err(),
                "Input '{}' should fail to parse, but it succeeded",
                input
            );
        }
    }

    #[test]
    fn test_very_large_expression() {
        let input = (0..100)
            .map(|i| format!("price{} > {}", i, i * 10))
            .collect::<Vec<_>>()
            .join(" AND ");

        let ast = LogicParser::parse_expression(&input).unwrap();

        // Generate the expected AST structure programmatically
        let mut expected_ast = ASTNode::BinaryOperation {
            left: Box::new(ASTNode::Identifier("price0".to_string())),
            operator: Operator::GreaterThan,
            right: Box::new(ASTNode::Number(0.0)),
        };

        for i in 1..100 {
            expected_ast = ASTNode::LogicalOperation {
                left: Box::new(expected_ast),
                operator: LogicalOperator::And,
                right: Box::new(ASTNode::BinaryOperation {
                    left: Box::new(ASTNode::Identifier(format!("price{}", i))),
                    operator: Operator::GreaterThan,
                    right: Box::new(ASTNode::Number((i * 10) as f64)),
                }),
            };
        }

        // Assert that the parsed AST matches the expected AST
        assert_eq!(ast, expected_ast);
    }
}
