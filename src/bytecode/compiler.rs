use crate::bytecode::{Bytecode, OpCode};
use log::debug;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "bytecode.expr.pest"] // Ensure this file contains the provided grammar
pub struct ExpressionParser;

pub struct BytecodeCompiler;

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&mut self, expression: &str) -> Result<Vec<u8>, String> {
        let mut bytecode = Vec::new();
        debug!("expression: {:?}", expression);
        let pairs = ExpressionParser::parse(Rule::expression, expression)
            .map_err(|e| format!("Parse error: {}", e))?;

        debug!("pairs: {:#?}", pairs);
        for pair in pairs {
            self.compile_expression(pair, &mut bytecode)?;
        }

        Ok(bytecode)
    }

    fn compile_expression(
        &mut self,
        pair: pest::iterators::Pair<Rule>,
        bytecode: &mut Vec<u8>,
    ) -> Result<(), String> {
        match pair.as_rule() {
            Rule::EOI => bytecode.push(OpCode::NoOp as u8),
            Rule::logical_expression | Rule::or_expression | Rule::and_expression => {
                let mut inner = pair.clone().into_inner();
                debug!("logical_expression: {:#?}", inner);
                self.compile_expression(inner.next().unwrap(), bytecode)?;
                for operand in inner {
                    self.compile_expression(operand, bytecode)?;
                    debug!("push op: {:#?}", pair.as_rule());
                    match pair.as_rule() {
                        Rule::or_expression => bytecode.push(OpCode::Or as u8),
                        Rule::and_expression => bytecode.push(OpCode::And as u8),
                        _ => {}
                    }
                }
            }
            Rule::not_expression => {
                let mut inner = pair.clone().into_inner();

                let not_operator_or_comparison = inner.next().unwrap();
                debug!(
                    "not_operator_or_comparison: {:#?}",
                    not_operator_or_comparison
                );
                if let Some(operand) = inner.next() {
                    self.compile_expression(operand, bytecode)?;
                }
                self.compile_expression(not_operator_or_comparison, bytecode)?;
            }
            Rule::NOT => {
                debug!("push not");
                bytecode.push(OpCode::Not as u8);
            }
            Rule::comparison_expression => {
                let mut inner = pair.into_inner();
                self.compile_expression(inner.next().unwrap(), bytecode)?;
                while let Some(op) = inner.next() {
                    let next_expr = inner.next().unwrap();
                    self.compile_expression(next_expr, bytecode)?;
                    debug!("push comparison opeartor: {:#?}", op.as_str());
                    match op.as_str() {
                        "==" => bytecode.push(OpCode::Eq as u8),
                        "!=" => bytecode.push(OpCode::Ne as u8),
                        ">" => bytecode.push(OpCode::Gt as u8),
                        "<" => bytecode.push(OpCode::Lt as u8),
                        ">=" => bytecode.push(OpCode::Ge as u8),
                        "<=" => bytecode.push(OpCode::Le as u8),
                        _ => return Err("Invalid comparison operator".to_string()),
                    }
                }
            }
            Rule::arithmetic_expression | Rule::exponent | Rule::term | Rule::factor => {
                let mut inner = pair.into_inner();
                self.compile_expression(inner.next().unwrap(), bytecode)?;
                while let Some(op) = inner.next() {
                    let next_expr = inner.next().unwrap();
                    self.compile_expression(next_expr, bytecode)?;
                    debug!("push arithmetic operator: {:#?}", op.as_str());
                    match op.as_str() {
                        "+" => bytecode.push(OpCode::Add as u8),
                        "-" => bytecode.push(OpCode::Sub as u8),
                        "*" => bytecode.push(OpCode::Mul as u8),
                        "/" => bytecode.push(OpCode::Div as u8),
                        "%" => bytecode.push(OpCode::Mod as u8),
                        "^" => bytecode.push(OpCode::Pow as u8),
                        _ => return Err("Invalid arithmetic operator".to_string()),
                    }
                }
            }
            Rule::function_call => {
                let mut inner = pair.into_inner();
                let func_name = inner.next().unwrap().as_str().to_string();
                let args: Vec<_> = inner.collect();

                let mut arg_count = 0;
                for arg in args {
                    // Only parse value.
                    // TODO: Maybe just use unnamed args
                    if arg.as_rule() != Rule::identifier {
                        self.compile_expression(arg, bytecode)?;
                        arg_count += 1;
                    }
                }

                debug!("push function_call");
                bytecode.push(OpCode::Call as u8);
                debug!("push arg_count: {arg_count}");
                bytecode.push(arg_count as u8);
                debug!("push func_name: {func_name}");
                bytecode.extend(func_name.as_bytes());
                bytecode.push(0); // Null terminator
            }
            Rule::property_access => {
                let mut inner = pair.into_inner();
                self.compile_expression(inner.next().unwrap(), bytecode)?;
                for property in inner {
                    let property_name = property.as_str().to_string();
                    debug!("push GetProperty");
                    bytecode.push(OpCode::GetProperty as u8);
                    debug!("push property_name: {property_name}");
                    bytecode.extend(property_name.as_bytes());
                    bytecode.push(0); // Null terminator
                }
            }
            Rule::number => {
                let value: f64 = pair
                    .as_str()
                    .parse()
                    .map_err(|_| "Invalid number".to_string())?;
                bytecode.push(OpCode::PushFloat as u8);
                bytecode.extend(&value.to_le_bytes()); // Store as little-endian bytes
            }
            Rule::boolean => {
                let value = if pair.as_str().to_lowercase() == "true" {
                    1
                } else {
                    0
                };
                debug!("push bool: {:#?}", value);
                bytecode.push(OpCode::PushBool as u8);
                bytecode.push(value);
            }
            Rule::identifier => {
                let var_name = pair.as_str().to_string();
                debug!("load variable");
                bytecode.push(OpCode::LoadVariable as u8);
                debug!("push var_name: {var_name}");
                bytecode.extend(var_name.as_bytes());
                bytecode.push(0);
            }
            Rule::group => {
                self.compile_expression(pair.into_inner().next().unwrap(), bytecode)?;
            }
            Rule::value => {
                self.compile_expression(pair.into_inner().next().unwrap(), bytecode)?;
            }
            _ => return Err(format!("Unhandled rule: {:?}", pair.as_rule())),
        }

        Ok(())
    }
}

// mod tests {
//     use super::*;
//
//     fn compile(expression: &str) -> Result<Vec<u8>, String> {
//         let mut compiler = BytecodeCompiler::new();
//         compiler.compile(expression)
//     }
//
//     #[test]
//     fn test_nested_expressions() {
//         let expr = "((3 + 2) * (8 - 4)) / 2";
//         let bytecode = compile(expr).unwrap();
//         assert_eq!(
//             bytecode,
//             vec![
//                 Bytecode::PushInt(3),
//                 Bytecode::PushInt(2),
//                 Bytecode::Add,
//                 Bytecode::PushInt(8),
//                 Bytecode::PushInt(4),
//                 Bytecode::Sub,
//                 Bytecode::Mul,
//                 Bytecode::PushInt(2),
//                 Bytecode::Div,
//             ]
//         );
//     }
//
//     #[test]
//     fn test_large_expression() {
//         let expr = "((10 - 5) * 2 + (8 / 4) - 1) * 3";
//         let bytecode = compile(expr).unwrap();
//         assert_eq!(
//             bytecode,
//             vec![
//                 Bytecode::PushInt(10),
//                 Bytecode::PushInt(5),
//                 Bytecode::Sub,
//                 Bytecode::PushInt(2),
//                 Bytecode::Mul,
//                 Bytecode::PushInt(8),
//                 Bytecode::PushInt(4),
//                 Bytecode::Div,
//                 Bytecode::Add,
//                 Bytecode::PushInt(1),
//                 Bytecode::Sub,
//                 Bytecode::PushInt(3),
//                 Bytecode::Mul,
//             ]
//         );
//     }
//
//     #[test]
//     fn test_function_call_with_property_access() {
//         let expr = "price_data.average().high";
//         let bytecode = compile(expr).unwrap();
//         assert_eq!(
//             bytecode,
//             vec![
//                 Bytecode::LoadVariable("price_data".into()),
//                 Bytecode::Call("average".into(), 0),
//                 Bytecode::GetProperty("high".into()),
//             ]
//         );
//     }
//
//     #[test]
//     fn test_division_by_zero() {
//         let expr = "10 / 0";
//         let bytecode = compile(expr).unwrap();
//         let mut executor = BytecodeExecutor::new();
//         let result = executor.execute(bytecode);
//         assert!(matches!(result, Err(_))); // Expect an error
//     }
//
//     #[test]
//     fn test_invalid_function_call() {
//         let expr = "unknown_func(3, 5)";
//         let bytecode = compile(expr).unwrap();
//         let mut executor = BytecodeExecutor::new();
//         let result = executor.execute(bytecode);
//         assert!(matches!(result, Err(_))); // Expect an error
//     }
// }
