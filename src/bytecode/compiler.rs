use crate::bytecode::Bytecode; // Assuming Bytecode and Value are defined as per your provided code
use log::debug;
use pest::Parser;
use pest_derive::Parser;
// use std::collections::{HashMap, HashSet};

#[derive(Parser)]
#[grammar = "bytecode.expr.pest"] // Ensure this file contains the provided grammar
pub struct ExpressionParser;

pub struct BytecodeCompiler {}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&mut self, expression: &str) -> Result<Vec<Bytecode>, String> {
        let mut bytecode = Vec::new();
        let pairs = ExpressionParser::parse(Rule::expression, expression)
            .map_err(|e| format!("Parse error: {}", e))?;

        for pair in pairs {
            self.compile_expression(pair, &mut bytecode)?;
        }

        Ok(bytecode)
    }

    fn compile_expression(
        &mut self,
        pair: pest::iterators::Pair<Rule>,
        bytecode: &mut Vec<Bytecode>,
    ) -> Result<(), String> {
        match pair.as_rule() {
            Rule::EOI => bytecode.push(Bytecode::NoOp),

            Rule::logical_expression | Rule::or_expression | Rule::and_expression => {
                let mut inner = pair.clone().into_inner();
                self.compile_expression(inner.next().unwrap(), bytecode)?;
                for operand in inner {
                    self.compile_expression(operand, bytecode)?;
                    match pair.as_rule() {
                        Rule::or_expression => bytecode.push(Bytecode::Or),
                        Rule::and_expression => bytecode.push(Bytecode::And),
                        _ => {}
                    }
                }
            }

            Rule::not_expression => {
                let mut inner = pair.clone().into_inner();
                let not_operator_or_comparison = inner.next().unwrap();
                if let Some(operand) = inner.next() {
                    self.compile_expression(operand, bytecode)?;
                }
                self.compile_expression(not_operator_or_comparison, bytecode)?;
            }

            Rule::NOT => {
                debug!("push not");
                bytecode.push(Bytecode::Not);
            }

            Rule::comparison_expression => {
                let mut inner = pair.into_inner();
                self.compile_expression(inner.next().unwrap(), bytecode)?;
                while let Some(op) = inner.next() {
                    let next_expr = inner.next().unwrap();
                    self.compile_expression(next_expr, bytecode)?;
                    match op.as_str() {
                        "==" => bytecode.push(Bytecode::Eq),
                        "!=" => bytecode.push(Bytecode::Ne),
                        ">" => bytecode.push(Bytecode::Gt),
                        "<" => bytecode.push(Bytecode::Lt),
                        ">=" => bytecode.push(Bytecode::Ge),
                        "<=" => bytecode.push(Bytecode::Le),
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
                    match op.as_str() {
                        "+" => bytecode.push(Bytecode::Add),
                        "-" => bytecode.push(Bytecode::Sub),
                        "*" => bytecode.push(Bytecode::Mul),
                        "/" => bytecode.push(Bytecode::Div),
                        "%" => bytecode.push(Bytecode::Mod),
                        "^" => bytecode.push(Bytecode::Pow),
                        _ => return Err("Invalid arithmetic operator".to_string()),
                    }
                }
            }

            Rule::function_call => {
                let mut inner = pair.into_inner();
                let fn_name = inner.next().unwrap().as_str().to_string();
                let args: Vec<_> = inner.collect();

                let mut arg_count = 0;
                for arg in args {
                    if arg.as_rule() != Rule::identifier {
                        self.compile_expression(arg, bytecode)?;
                        arg_count += 1;
                    }
                }

                // let fn_addr: fn(&[Value]) -> Result<Value, String> =
                //     *(self.functions.get(&func_name).unwrap());

                bytecode.push(Bytecode::Call(fn_name, arg_count));
            }

            Rule::property_access => {
                let mut inner = pair.into_inner();
                self.compile_expression(inner.next().unwrap(), bytecode)?;
                for property in inner {
                    let property_name = property.as_str().to_string();
                    bytecode.push(Bytecode::GetProperty(property_name));
                }
            }

            Rule::number => {
                let value: f64 = pair
                    .as_str()
                    .parse()
                    .map_err(|_| "Invalid number".to_string())?;
                bytecode.push(Bytecode::PushFloat(value));
            }

            Rule::boolean => {
                let value = if pair.as_str().to_lowercase() == "true" {
                    true
                } else {
                    false
                };
                bytecode.push(Bytecode::PushBool(value));
            }

            Rule::identifier => {
                let var_name = pair.as_str().to_string();
                bytecode.push(Bytecode::LoadVariable(var_name));
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
