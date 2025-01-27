use crate::ast::{ASTNode, FunctionArgValue, FunctionArgs, FunctionResult, Parser};
use std::collections::HashMap;
use std::sync::Arc;

pub type Function = Arc<dyn Fn(&FunctionArgs) -> Result<FunctionResult, String> + Send + Sync>;

pub struct Evaluator {
    pub(crate) functions: HashMap<String, Function>,
}

impl Evaluator {
    /// Creates a new `Evaluator` with a given maximum cache size.
    pub fn new(_max_cache_size: usize) -> Self {
        Self {
            functions: HashMap::new(),
        }
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
    pub fn evaluate_expression(
        &mut self,
        expression: &str,
        context: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        // Step 1: Parse the expression into an AST
        let ast = self.parse_expression(expression)?;

        // Step 2: Resolve identifiers in the AST
        let resolved_ast = ast.resolve_identifiers(context)?;

        // Step 3: Evaluate the resolved AST
        self.evaluate_ast(&resolved_ast, context)
    }

    /// Evaluate a single AST node against a single context.
    pub fn evaluate_ast(
        &mut self,
        ast: &ASTNode,
        context: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        let resolved_ast = ast.resolve_identifiers(context)?; // Resolve identifiers per context.
        self.evaluate(&resolved_ast, context) // Evaluate the resolved AST.
    }

    /// Registers a function with the evaluator.
    pub fn register_function<F>(&mut self, name: &str, function: F)
    where
        F: Fn(&FunctionArgs) -> Result<FunctionResult, String> + Send + Sync + 'static,
    {
        self.functions.insert(name.to_string(), Arc::new(function));
    }

    /// Evaluates an `ASTNode` with a given context.
    pub fn evaluate(
        &mut self,
        ast: &ASTNode,
        context: &HashMap<String, f64>,
    ) -> Result<f64, String> {
        // Evaluate the AST node
        let result = match ast {
            ASTNode::Number(n) => Ok(*n),

            ASTNode::Identifier(ident) => context
                .get(ident)
                .copied()
                .ok_or_else(|| format!("Identifier '{}' not found in context", ident)),

            ASTNode::BinaryOperation {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left, context)?;
                // assert_eq!(left_value, 200.0);
                let right_value = self.evaluate(right, context)?;
                // assert_eq!(right_value, 200.0);
                operator.apply(left_value, right_value)
            }

            ASTNode::LogicalOperation {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left, context)?;
                let right_value = self.evaluate(right, context)?;
                operator.apply(left_value, right_value)
            }

            ASTNode::NotOperation(inner) => {
                Ok((self.evaluate(inner, context)? == 0.0) as i32 as f64)
            }

            ASTNode::FunctionCall { name, args } => {
                // Evaluate the function
                let function = self
                    .functions
                    .get(name)
                    .ok_or_else(|| format!("Function {} not registered", name))?;

                // Evaluate the arguments, resolving identifiers to values from the context
                let mut new_args = args.clone();
                for (arg_name, arg_value) in args.args.iter() {
                    let resolved_value: FunctionArgValue = match arg_value {
                        FunctionArgValue::Identifier(ident) => {
                            // Resolve the identifier to a value in the context
                            context
                                .get(ident)
                                .copied()
                                .ok_or_else(|| {
                                    format!("Identifier '{}' not found in context", ident)
                                })?
                                .try_into()
                                .unwrap()
                        }
                        _ => arg_value.clone(),
                    };

                    new_args.insert(&arg_name, resolved_value);
                }

                // Call the function with the resolved arguments
                let result = function(&new_args)?;

                match result {
                    FunctionResult::UnnamedF64(value) => Ok(value),
                    FunctionResult::NamedF64Map(_) => {
                        Err("Expected single value, got multi-value".to_string())
                    }
                }
            }
            ASTNode::PropertyAccess { base, property } => {
                if let ASTNode::FunctionCall { name, args } = &**base {
                    let function = self
                        .functions
                        .get(name)
                        .ok_or_else(|| format!("Function {} not registered", name))?;
                    if let FunctionResult::NamedF64Map(map) = function(args)? {
                        map.get(property)
                            .copied()
                            .ok_or_else(|| format!("Property {} not found in result", property))
                    } else {
                        Err("Expected multi-value, got single value".to_string())
                    }
                } else {
                    Err("Base must be a function call".to_string())
                }
            }
            ASTNode::Group(inner) => self.evaluate(inner, context),
        }?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_evaluator() -> Evaluator {
        let mut evaluator = Evaluator::new(100);

        // Register a sample function
        evaluator.register_function("add", |args| {
            let a = args.get_number("a")?;
            let b = args.get_number("b")?;

            Ok(FunctionResult::UnnamedF64(a + b))
        });

        evaluator.register_function("sum", |args| {
            let a = args.get_number("a").unwrap();
            Ok(FunctionResult::UnnamedF64(a * 2.0))
        });

        evaluator
    }

    #[test]
    fn test_simple_expression() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 100.0)]);

        let result = evaluator.evaluate_expression("price > 50", &context);
        assert_eq!(result.unwrap(), 1.0); // True
    }

    #[test]
    fn test_complex_expression() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 100.0), ("volume".to_string(), 5000.0)]);

        let result = evaluator.evaluate_expression("price > 50 AND volume < 6000", &context);
        assert_eq!(result.unwrap(), 1.0); // True
    }

    #[test]
    fn test_nested_expression() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([
            ("price".to_string(), 100.0),
            ("volume".to_string(), 5000.0),
            ("threshold".to_string(), 6000.0),
        ]);

        let result = evaluator.evaluate_expression(
            "(price > 50 AND volume < threshold) OR volume >= 3000",
            &context,
        );
        assert_eq!(result.unwrap(), 1.0); // True
    }

    #[test]
    fn test_grouped_expression() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 100.0)]);

        let result = evaluator.evaluate_expression("(price > 50) AND (price < 150)", &context);
        assert_eq!(result.unwrap(), 1.0); // True
    }

    #[test]
    fn test_function_call() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::new();

        let result = evaluator.evaluate_expression("add(a: 10, b: 20)", &context);
        assert_eq!(result.unwrap(), 30.0); // Result of add(10, 20)
    }

    #[test]
    fn test_property_access() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::new();

        evaluator.register_function("test_func", |_args| {
            Ok(FunctionResult::NamedF64Map(
                [("result".to_string(), 42.0)].iter().cloned().collect(),
            ))
        });

        let result = evaluator.evaluate_expression("test_func().result", &context);
        assert_eq!(result.unwrap(), 42.0);
    }

    #[test]
    fn test_complex_arithmetic() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 100.0), ("volume".to_string(), 20.0)]);

        let result = evaluator.evaluate_expression("price + 20 * volume", &context);
        assert_eq!(result.unwrap(), 500.0); // 100 + 20*20
    }

    #[test]
    fn test_complex_logical_evaluation() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([
            ("price".to_string(), 100.0),
            ("volume".to_string(), 5000.0),
            ("threshold".to_string(), 6000.0),
        ]);

        let result = evaluator.evaluate_expression(
            "price > 50 AND (volume < threshold OR volume >= 3000)",
            &context,
        );
        assert_eq!(result.unwrap(), 1.0); // True
    }

    #[test]
    fn test_function_with_logic() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 100.0)]);

        let result = evaluator.evaluate_expression("sum(a: price) > 100", &context);
        assert_eq!(result.unwrap(), 1.0); // True, because sum(price) = price * 2 = 200 > 100
    }

    #[test]
    fn test_simple_binary_expression() {
        let mut evaluator = Evaluator::new(100);
        let context = HashMap::from([("price".to_string(), 100.0), ("volume".to_string(), 50.0)]);

        assert_eq!(
            evaluator
                .evaluate_expression("price + volume", &context)
                .unwrap(),
            150.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("price - volume", &context)
                .unwrap(),
            50.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("price * volume", &context)
                .unwrap(),
            5000.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("price / volume", &context)
                .unwrap(),
            2.0
        );
    }

    #[test]
    fn test_complex_logical_expression() {
        let mut evaluator = Evaluator::new(100);
        let context = HashMap::from([("price".to_string(), 100.0), ("volume".to_string(), 50.0)]);

        assert_eq!(
            evaluator
                .evaluate_expression("price > 90 AND volume < 100", &context)
                .unwrap(),
            1.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("price > 100 OR volume <= 50", &context)
                .unwrap(),
            1.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("NOT (price == 100)", &context)
                .unwrap(),
            0.0
        );
    }

    #[test]
    fn test_function_call_2() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 100.0), ("volume".to_string(), 50.0)]);

        assert_eq!(
            evaluator
                .evaluate_expression("add(a: price, b: volume)", &context)
                .unwrap(),
            150.0
        );
    }

    #[test]
    fn test_nested_and_grouped_expression() {
        let mut evaluator = setup_evaluator();
        let context = HashMap::from([("price".to_string(), 100.0), ("volume".to_string(), 50.0)]);

        assert_eq!(
            evaluator
                .evaluate_expression("(price + volume) * 2", &context)
                .unwrap(),
            300.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("(price > 90 AND volume < 100) OR price == 50", &context)
                .unwrap(),
            1.0
        );
    }

    #[test]
    fn test_property_access_2() {
        let mut evaluator = setup_evaluator();
        evaluator.register_function("get_metrics", |_args| {
            Ok(FunctionResult::NamedF64Map(HashMap::from([
                ("price".to_string(), 100.0),
                ("volume".to_string(), 50.0),
            ])))
        });

        assert_eq!(
            evaluator
                .evaluate_expression("get_metrics().price", &HashMap::new())
                .unwrap(),
            100.0
        );
        assert_eq!(
            evaluator
                .evaluate_expression("get_metrics().volume", &HashMap::new())
                .unwrap(),
            50.0
        );
    }

    #[test]
    fn test_error_cases() {
        let mut evaluator = setup_evaluator();

        // Undefined identifier
        assert!(evaluator
            .evaluate_expression("undefined_variable", &HashMap::new())
            .is_err());

        // Undefined function
        assert!(evaluator
            .evaluate_expression("undefined_function()", &HashMap::new())
            .is_err());

        // Invalid expression
        assert!(evaluator
            .evaluate_expression("price + ", &HashMap::new())
            .is_err());
    }
}
