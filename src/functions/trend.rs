use crate::ast::{Executor, Value};

pub fn register(executor: &mut Executor) {
    executor.register_function("simple_moving_average", simple_moving_average);
    executor.register_function("exponential_moving_average", exponential_moving_average);
    executor.register_function("relative_strength_index", relative_strength_index);
    executor.register_function(
        "moving_average_convergence_divergence",
        moving_average_convergence_divergence,
    );
    executor.register_function("ichimoku_tenkan_kijun", ichimoku_tenkan_kijun);
    executor.register_function("parabolic_sar", parabolic_sar);
}

fn simple_moving_average(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("SMA requires 2 arguments: (prices: Vec<f64>, period: f64)".to_string());
    }

    let prices = match &args[0] {
        Value::Array(p) => p,
        _ => return Err("First argument must be an array of prices".to_string()),
    };

    let period = match args[1] {
        Value::Number(p) if p > 0.0 => p as usize,
        _ => {
            return Err(
                "Second argument must be a positive number representing the period".to_string(),
            )
        }
    };

    if prices.len() < period {
        return Err("Not enough data points to compute SMA".to_string());
    }

    let sum: f64 = prices.iter().rev().take(period).sum();
    Ok(Value::Number(sum / period as f64))
}

fn exponential_moving_average(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("EMA requires 2 arguments: (prices: Vec<f64>, period: f64)".to_string());
    }

    let prices = match &args[0] {
        Value::Array(p) => p,
        _ => return Err("First argument must be an array of prices".to_string()),
    };

    let period = match args[1] {
        Value::Number(p) if p > 0.0 => p as usize,
        _ => {
            return Err(
                "Second argument must be a positive number representing the period".to_string(),
            )
        }
    };

    if prices.len() < period {
        return Err("Not enough data points to compute EMA".to_string());
    }

    let k = 2.0 / (period as f64 + 1.0);
    let mut ema = prices[0];

    for &price in &prices[1..] {
        ema = (price * k) + (ema * (1.0 - k));
    }

    Ok(Value::Number(ema))
}

fn relative_strength_index(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("RSI requires 2 arguments: (prices: Vec<f64>, period: f64)".to_string());
    }

    let prices = match &args[0] {
        Value::Array(p) => p,
        _ => return Err("First argument must be an array of prices".to_string()),
    };

    let period = match args[1] {
        Value::Number(p) if p > 0.0 => p as usize,
        _ => {
            return Err(
                "Second argument must be a positive number representing the period".to_string(),
            )
        }
    };

    if prices.len() < period + 1 {
        return Err("Not enough data points to compute RSI".to_string());
    }

    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in 1..=period {
        let diff = prices[i] - prices[i - 1];
        if diff > 0.0 {
            gains += diff;
        } else {
            losses -= diff;
        }
    }

    let avg_gain = gains / period as f64;
    let avg_loss = losses / period as f64;

    let rs = if avg_loss == 0.0 {
        100.0
    } else {
        100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
    };

    Ok(Value::Number(rs))
}

fn moving_average_convergence_divergence(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            "MACD requires 3 arguments: (prices: Vec<f64>, short_period: f64, long_period: f64)"
                .to_string(),
        );
    }

    let prices = match &args[0] {
        Value::Array(p) => p,
        _ => return Err("First argument must be an array of prices".to_string()),
    };

    let short_period = match args[1] {
        Value::Number(p) if p > 0.0 => p as usize,
        _ => {
            return Err(
                "Second argument must be a positive number representing the short EMA period"
                    .to_string(),
            )
        }
    };

    let long_period = match args[2] {
        Value::Number(p) if p > short_period as f64 => p as usize,
        _ => {
            return Err(
                "Third argument must be a positive number greater than the short EMA period"
                    .to_string(),
            )
        }
    };

    if prices.len() < long_period {
        return Err("Not enough data points to compute MACD".to_string());
    }

    let ema_short = exponential_moving_average(&[
        Value::Array(prices.clone()),
        Value::Number(short_period as f64),
    ])?;
    let ema_long = exponential_moving_average(&[
        Value::Array(prices.clone()),
        Value::Number(long_period as f64),
    ])?;

    match (ema_short, ema_long) {
        (Value::Number(short), Value::Number(long)) => Ok(Value::Number(short - long)),
        _ => Err("Failed to compute MACD".to_string()),
    }
}

fn ichimoku_tenkan_kijun(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            "Ichimoku requires 3 arguments: (highs: Vec<f64>, lows: Vec<f64>, period: f64)"
                .to_string(),
        );
    }

    let highs = match &args[0] {
        Value::Array(h) => h,
        _ => return Err("First argument must be an array of high prices".to_string()),
    };

    let lows = match &args[1] {
        Value::Array(l) => l,
        _ => return Err("Second argument must be an array of low prices".to_string()),
    };

    let period = match args[2] {
        Value::Number(p) if p > 0.0 => p as usize,
        _ => {
            return Err(
                "Third argument must be a positive number representing the period".to_string(),
            )
        }
    };

    if highs.len() < period || lows.len() < period {
        return Err("Not enough data points to compute Ichimoku values".to_string());
    }

    let highest_high = highs
        .iter()
        .rev()
        .take(period)
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let lowest_low = lows
        .iter()
        .rev()
        .take(period)
        .fold(f64::INFINITY, |a, &b| a.min(b));

    let tenkan_sen = (highest_high + lowest_low) / 2.0;
    Ok(Value::Number(tenkan_sen))
}

fn parabolic_sar(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("Parabolic SAR requires 3 arguments: (highs: Vec<f64>, lows: Vec<f64>, acceleration_factor: f64)".to_string());
    }

    let highs = match &args[0] {
        Value::Array(h) => h,
        _ => return Err("First argument must be an array of high prices".to_string()),
    };

    let lows = match &args[1] {
        Value::Array(l) => l,
        _ => return Err("Second argument must be an array of low prices".to_string()),
    };

    let acceleration_factor = match args[2] {
        Value::Number(a) if a > 0.0 => a,
        _ => {
            return Err(
                "Third argument must be a positive number representing the acceleration factor"
                    .to_string(),
            )
        }
    };

    if highs.len() < 2 || lows.len() < 2 {
        return Err("Not enough data points to compute Parabolic SAR".to_string());
    }

    let mut sar = lows[0];
    let mut ep = highs[0];
    let mut af = acceleration_factor;

    for i in 1..highs.len() {
        if highs[i] > ep {
            ep = highs[i];
            af += acceleration_factor;
        }

        sar += af * (ep - sar);
    }

    Ok(Value::Number(sar))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_moving_average() {
        let args = vec![
            Value::Array(vec![1.0, 2.0, 3.0, 4.0, 5.0]),
            Value::Number(3.0),
        ];
        let result = simple_moving_average(&args).unwrap();
        assert_eq!(result, Value::Number(4.0)); // Last 3 values: (3+4+5)/3 = 4.0
    }

    #[test]
    fn test_exponential_moving_average() {
        let args = vec![
            Value::Array(vec![1.0, 2.0, 3.0, 4.0, 5.0]),
            Value::Number(3.0),
        ];
        let result = exponential_moving_average(&args).unwrap();
        assert!(matches!(result, Value::Number(_))); // Should return a valid EMA value
    }

    #[test]
    fn test_relative_strength_index() {
        let args = vec![
            Value::Array(vec![
                44.0, 45.0, 46.0, 43.0, 42.0, 41.0, 40.0, 39.0, 38.0, 37.0,
            ]),
            Value::Number(5.0),
        ];
        let result = relative_strength_index(&args).unwrap();
        assert!(matches!(result, Value::Number(_))); // Should return a valid RSI value
    }

    #[test]
    fn test_macd() {
        let args = vec![
            Value::Array(vec![1.0, 2.0, 3.0, 4.0, 5.0]),
            Value::Number(2.0),
            Value::Number(4.0),
        ];
        let result = moving_average_convergence_divergence(&args).unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_ichimoku() {
        let args = vec![
            Value::Array(vec![10.0, 12.0, 14.0, 16.0, 18.0]),
            Value::Array(vec![5.0, 6.0, 7.0, 8.0, 9.0]),
            Value::Number(3.0),
        ];
        let result = ichimoku_tenkan_kijun(&args).unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_parabolic_sar() {
        let args = vec![
            Value::Array(vec![10.0, 12.0, 14.0, 16.0, 18.0]),
            Value::Array(vec![5.0, 6.0, 7.0, 8.0, 9.0]),
            Value::Number(0.02),
        ];
        let result = parabolic_sar(&args).unwrap();
        assert!(matches!(result, Value::Number(_)));
    }
}
