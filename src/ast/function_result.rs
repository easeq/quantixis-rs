use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionResult {
    UnnamedF64(f64),
    NamedF64Map(HashMap<String, f64>),
}

impl FunctionResult {
    /// Utility function to convert `FunctionResult` to a single `f64` if applicable.
    pub fn as_number(&self) -> Option<f64> {
        if let FunctionResult::UnnamedF64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Utility function to convert `FunctionResult` to a `HashMap` if applicable.
    pub fn as_map(&self) -> Option<&HashMap<String, f64>> {
        if let FunctionResult::NamedF64Map(map) = self {
            Some(map)
        } else {
            None
        }
    }
}
