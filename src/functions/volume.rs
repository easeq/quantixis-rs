use crate::ast::{Executor, Value};
use quantixis_macros::quantinxis_fn;

pub fn register(executor: &mut Executor) {
    executor.register_function("on_balance_volume", on_balance_volume);
    executor.register_function("chaikin_money_flow", chaikin_money_flow);
}

#[quantinxis_fn]
fn on_balance_volume(prices: Vec<f64>, volumes: Vec<f64>) -> Result<Value, String> {
    if prices.len() != volumes.len() {
        return Err("Prices and volumes arrays must have the same length".to_string());
    }

    let mut obv = 0.0;
    for i in 1..prices.len() {
        if prices[i] > prices[i - 1] {
            obv += volumes[i];
        } else if prices[i] < prices[i - 1] {
            obv -= volumes[i];
        }
    }

    Ok(Value::Number(obv))
}

#[quantinxis_fn]
fn chaikin_money_flow(
    highs: Vec<f64>,
    lows: Vec<f64>,
    closes: Vec<f64>,
    volumes: Vec<f64>,
) -> Result<Value, String> {
    if highs.len() != lows.len() || lows.len() != closes.len() || closes.len() != volumes.len() {
        return Err("All input arrays must have the same length".to_string());
    }

    let mut money_flow_volume = 0.0;
    let mut total_volume = 0.0;

    for i in 0..highs.len() {
        let money_flow_multiplier =
            ((closes[i] - lows[i]) - (highs[i] - closes[i])) / (highs[i] - lows[i]).max(0.00001);
        money_flow_volume += money_flow_multiplier * volumes[i];
        total_volume += volumes[i];
    }

    let cmf = if total_volume != 0.0 {
        money_flow_volume / total_volume
    } else {
        0.0
    };

    Ok(Value::Number(cmf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obv() {
        let args = vec![
            Value::Array(vec![10.0, 11.0, 12.0, 11.0, 12.0]),
            Value::Array(vec![1000.0, 1200.0, 1500.0, 1300.0, 1400.0]),
        ];
        let result = on_balance_volume(&args).unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_cmf() {
        let args = vec![
            Value::Array(vec![10.0, 12.0, 14.0, 16.0, 18.0]),
            Value::Array(vec![5.0, 6.0, 7.0, 8.0, 9.0]),
            Value::Array(vec![7.0, 9.0, 11.0, 13.0, 15.0]),
            Value::Array(vec![1000.0, 1200.0, 1500.0, 1300.0, 1400.0]),
        ];
        let result = chaikin_money_flow(&args).unwrap();
        assert!(matches!(result, Value::Number(_)));
    }
}
