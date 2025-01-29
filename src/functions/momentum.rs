use crate::ast::{Executor, Value};

pub fn register(executor: &mut Executor) {
    executor.register_function("rate_of_change", rate_of_change);
    executor.register_function("stochastic", stochastic);
    executor.register_function("momentum", momentum);
    executor.register_function("commodity_channel_index", commodity_channel_index);
    executor.register_function("chande_momentum_oscillator", chande_momentum_oscillator);
    executor.register_function("relative_vigor_index", relative_vigor_index);
    executor.register_function("williams_percent_r", williams_percent_r);
    executor.register_function("awesome_osc", awesome_oscillator);
    executor.register_function("ad_oscillator", ad_oscillator);
    executor.register_function("klinger_oscillator", klinger_oscillator);
    executor.register_function("choppiness_index", choppiness_index);
}

fn rate_of_change(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("ROC requires 2 arguments: (prices: Vec<f64>, period: f64)".to_string());
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

    if prices.len() <= period {
        return Err("Not enough data points to compute ROC".to_string());
    }

    let roc = ((prices.last().unwrap() - prices[prices.len() - period - 1])
        / prices[prices.len() - period - 1])
        * 100.0;
    Ok(Value::Number(roc))
}

fn stochastic(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            "Stochastic requires 3 arguments: (highs: Vec<f64>, lows: Vec<f64>, closes: Vec<f64>)"
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

    let closes = match &args[2] {
        Value::Array(c) => c,
        _ => return Err("Third argument must be an array of closing prices".to_string()),
    };

    if highs.len() != lows.len() || highs.len() != closes.len() || highs.is_empty() {
        return Err("All arrays must have the same nonzero length".to_string());
    }

    let highest_high = highs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let lowest_low = lows.iter().cloned().fold(f64::INFINITY, f64::min);
    let latest_close = *closes.last().unwrap();

    let stochastic = if highest_high != lowest_low {
        ((latest_close - lowest_low) / (highest_high - lowest_low)) * 100.0
    } else {
        0.0
    };

    Ok(Value::Number(stochastic))
}

fn momentum(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("Momentum requires 2 arguments: (prices: Vec<f64>, period: f64)".to_string());
    }

    let prices = match &args[0] {
        Value::Array(p) => p,
        _ => return Err("First argument must be an array of prices".to_string()),
    };

    let period = match args[1] {
        Value::Number(p) if p > 0.0 => p as usize,
        _ => return Err("Second argument must be a positive number (period)".to_string()),
    };

    if prices.len() <= period {
        return Err("Not enough data points to compute Momentum".to_string());
    }

    let momentum = prices.last().unwrap() - prices[prices.len() - period - 1];
    Ok(Value::Number(momentum))
}

fn commodity_channel_index(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            "CCI requires 3 arguments: (highs: Vec<f64>, lows: Vec<f64>, closes: Vec<f64>)"
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

    let closes = match &args[2] {
        Value::Array(c) => c,
        _ => return Err("Third argument must be an array of closing prices".to_string()),
    };

    if highs.len() != lows.len() || highs.len() != closes.len() || highs.is_empty() {
        return Err("All arrays must have the same nonzero length".to_string());
    }

    let typical_prices: Vec<f64> = highs
        .iter()
        .zip(lows)
        .zip(closes)
        .map(|((&h, &l), &c)| (h + l + c) / 3.0)
        .collect();
    let mean_price = typical_prices.iter().sum::<f64>() / typical_prices.len() as f64;
    let mean_deviation = typical_prices
        .iter()
        .map(|&p| (p - mean_price).abs())
        .sum::<f64>()
        / typical_prices.len() as f64;

    let cci = if mean_deviation != 0.0 {
        (typical_prices.last().unwrap() - mean_price) / (0.015 * mean_deviation)
    } else {
        0.0
    };

    Ok(Value::Number(cci))
}

fn chande_momentum_oscillator(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("CMO requires 1 argument: (prices: Vec<f64>)".to_string());
    }

    let prices = match &args[0] {
        Value::Array(p) => p,
        _ => return Err("First argument must be an array of prices".to_string()),
    };

    if prices.len() < 2 {
        return Err("Not enough data points to compute CMO".to_string());
    }

    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains += change;
        } else {
            losses -= change;
        }
    }

    let cmo = if (gains + losses) != 0.0 {
        ((gains - losses) / (gains + losses)) * 100.0
    } else {
        0.0
    };

    Ok(Value::Number(cmo))
}

fn relative_vigor_index(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("RVI requires 2 arguments: (closes: Vec<f64>, opens: Vec<f64>)".to_string());
    }

    let closes = match &args[0] {
        Value::Array(c) => c,
        _ => return Err("First argument must be an array of closing prices".to_string()),
    };

    let opens = match &args[1] {
        Value::Array(o) => o,
        _ => return Err("Second argument must be an array of opening prices".to_string()),
    };

    if closes.len() != opens.len() || closes.is_empty() {
        return Err(
            "Closing and opening price arrays must have the same nonzero length".to_string(),
        );
    }

    let sum_close_open: f64 = closes.iter().zip(opens).map(|(c, o)| c - o).sum();
    let sum_high_low: f64 = closes.iter().zip(opens).map(|(c, o)| c + o).sum();

    let rvi = sum_close_open / sum_high_low;
    Ok(Value::Number(rvi))
}

fn williams_percent_r(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(
            "Williams %R requires 3 arguments: (highs: Vec<f64>, lows: Vec<f64>, closes: Vec<f64>)"
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

    let closes = match &args[2] {
        Value::Array(c) => c,
        _ => return Err("Third argument must be an array of closing prices".to_string()),
    };

    if highs.len() != lows.len() || highs.len() != closes.len() || highs.is_empty() {
        return Err("All arrays must have the same nonzero length".to_string());
    }

    let highest_high = highs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let lowest_low = lows.iter().cloned().fold(f64::INFINITY, f64::min);
    let latest_close = *closes.last().unwrap();

    let percent_r = if highest_high != lowest_low {
        ((highest_high - latest_close) / (highest_high - lowest_low)) * -100.0
    } else {
        0.0
    };

    Ok(Value::Number(percent_r))
}

fn awesome_oscillator(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("Awesome Oscillator requires 1 argument: (prices: Vec<f64>)".to_string());
    }

    let prices = match &args[0] {
        Value::Array(p) => p,
        _ => return Err("First argument must be an array of prices".to_string()),
    };

    if prices.len() < 34 {
        return Err("Not enough data points to compute Awesome Oscillator".to_string());
    }

    let sma_5 = prices.iter().rev().take(5).sum::<f64>() / 5.0;
    let sma_34 = prices.iter().rev().take(34).sum::<f64>() / 34.0;

    Ok(Value::Number(sma_5 - sma_34))
}

fn ad_oscillator(args: &[Value]) -> Result<Value, String> {
    if args.len() != 4 {
        return Err("AD Oscillator requires 4 arguments: (highs: Vec<f64>, lows: Vec<f64>, closes: Vec<f64>, volumes: Vec<f64>)".to_string());
    }

    let highs = match &args[0] {
        Value::Array(h) => h,
        _ => return Err("First argument must be an array of high prices".to_string()),
    };

    let lows = match &args[1] {
        Value::Array(l) => l,
        _ => return Err("Second argument must be an array of low prices".to_string()),
    };

    let closes = match &args[2] {
        Value::Array(c) => c,
        _ => return Err("Third argument must be an array of closing prices".to_string()),
    };

    let volumes = match &args[3] {
        Value::Array(v) => v,
        _ => return Err("Fourth argument must be an array of volumes".to_string()),
    };

    if highs.len() != lows.len()
        || highs.len() != closes.len()
        || highs.len() != volumes.len()
        || highs.is_empty()
    {
        return Err("All input arrays must have the same nonzero length".to_string());
    }

    let mut ad_line = 0.0;
    for i in 0..highs.len() {
        let money_flow_multiplier = if highs[i] != lows[i] {
            ((closes[i] - lows[i]) - (highs[i] - closes[i])) / (highs[i] - lows[i])
        } else {
            0.0
        };
        let money_flow_volume = money_flow_multiplier * volumes[i];
        ad_line += money_flow_volume;
    }
    Ok(Value::Number(ad_line))
}

fn klinger_oscillator(args: &[Value]) -> Result<Value, String> {
    if args.len() != 4 {
        return Err("Klinger Oscillator requires 4 arguments: (highs: Vec<f64>, lows: Vec<f64>, closes: Vec<f64>, volumes: Vec<f64>)".to_string());
    }

    let highs = match &args[0] {
        Value::Array(h) => h,
        _ => return Err("First argument must be an array of high prices".to_string()),
    };

    let lows = match &args[1] {
        Value::Array(l) => l,
        _ => return Err("Second argument must be an array of low prices".to_string()),
    };

    let closes = match &args[2] {
        Value::Array(c) => c,
        _ => return Err("Third argument must be an array of closing prices".to_string()),
    };

    let volumes = match &args[3] {
        Value::Array(v) => v,
        _ => return Err("Fourth argument must be an array of volumes".to_string()),
    };

    if highs.len() != lows.len()
        || highs.len() != closes.len()
        || highs.len() != volumes.len()
        || highs.is_empty()
    {
        return Err("All input arrays must have the same nonzero length".to_string());
    }

    let mut kvo = Vec::new();
    for i in 1..highs.len() {
        let volume_force = (volumes[i] * ((closes[i] - closes[i - 1]) / closes[i - 1])) as f64;
        kvo.push(volume_force);
    }

    let short_ema = kvo.iter().rev().take(34).sum::<f64>() / 34.0;
    let long_ema = kvo.iter().rev().take(55).sum::<f64>() / 55.0;

    Ok(Value::Number(short_ema - long_ema))
}

fn choppiness_index(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("Choppiness Index requires 3 arguments: (highs: Vec<f64>, lows: Vec<f64>, closes: Vec<f64>)".to_string());
    }

    let highs = match &args[0] {
        Value::Array(h) => h,
        _ => return Err("First argument must be an array of high prices".to_string()),
    };

    let lows = match &args[1] {
        Value::Array(l) => l,
        _ => return Err("Second argument must be an array of low prices".to_string()),
    };

    let closes = match &args[2] {
        Value::Array(c) => c,
        _ => return Err("Third argument must be an array of closing prices".to_string()),
    };

    if highs.len() != lows.len() || highs.len() != closes.len() || highs.is_empty() {
        return Err("All input arrays must have the same nonzero length".to_string());
    }

    let tr: f64 = highs.iter().zip(lows.iter()).map(|(h, l)| h - l).sum();

    let highest_high = highs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let lowest_low = lows.iter().cloned().fold(f64::INFINITY, f64::min);

    let choppiness = if tr != 0.0 {
        100.0 * ((tr.ln()) / (highest_high - lowest_low).ln())
    } else {
        0.0
    };

    Ok(Value::Number(choppiness))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roc() {
        let args = vec![
            Value::Array(vec![10.0, 11.0, 12.0, 13.0, 14.0]),
            Value::Number(2.0),
        ];
        let result = rate_of_change(&args).unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_stochastic_oscillator() {
        let result = stochastic(&[
            Value::Array(vec![10.0, 12.0, 15.0, 18.0, 20.0]), // closing prices
            Value::Array(vec![5.0, 6.0, 7.0, 8.0, 9.0]),      // lowest lows
            Value::Array(vec![15.0, 16.0, 17.0, 18.0, 19.0]), // highest highs
        ])
        .unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_momentum() {
        let result = momentum(&[
            Value::Array(vec![10.0, 12.0, 15.0, 18.0, 20.0]), // closing prices
            Value::Number(2.0),                               // period
        ])
        .unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_commodity_channel_index() {
        let result = commodity_channel_index(&[
            Value::Array(vec![10.0, 12.0, 15.0, 18.0, 20.0]), // highs
            Value::Array(vec![5.0, 6.0, 7.0, 8.0, 9.0]),      // lows
            Value::Array(vec![8.0, 10.0, 12.0, 14.0, 16.0]),  // closes
            Value::Number(3.0),                               // period
        ])
        .unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_chande_momentum_oscillator() {
        let result = chande_momentum_oscillator(&[
            Value::Array(vec![10.0, 12.0, 15.0, 18.0, 20.0]), // closing prices
            Value::Number(3.0),                               // period
        ])
        .unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_williams_percent_r() {
        let result = williams_percent_r(&[
            Value::Array(vec![10.0, 12.0, 15.0]),
            Value::Array(vec![5.0, 6.0, 7.0]),
            Value::Array(vec![8.0, 10.0, 12.0]),
        ])
        .unwrap();
        if let Value::Number(r) = result {
            assert!(r < 0.0);
        } else {
            panic!("Expected Number value");
        }
    }

    #[test]
    fn test_awesome_oscillator() {
        let result = awesome_oscillator(&[Value::Array(vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0,
        ])])
        .unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_ad_oscillator() {
        let result = ad_oscillator(&[
            Value::Array(vec![10.0, 12.0, 15.0]),
            Value::Array(vec![5.0, 6.0, 7.0]),
            Value::Array(vec![8.0, 10.0, 12.0]),
            Value::Array(vec![1000.0, 2000.0, 1500.0]),
        ])
        .unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_klinger_oscillator() {
        let result = klinger_oscillator(&[
            Value::Array(vec![10.0, 12.0, 15.0]),
            Value::Array(vec![5.0, 6.0, 7.0]),
            Value::Array(vec![8.0, 10.0, 12.0]),
            Value::Array(vec![1000.0, 2000.0, 1500.0]),
        ])
        .unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    fn test_choppiness_index() {
        let result = choppiness_index(&[
            Value::Array(vec![10.0, 12.0, 15.0]),
            Value::Array(vec![5.0, 6.0, 7.0]),
            Value::Array(vec![8.0, 10.0, 12.0]),
        ])
        .unwrap();
        assert!(matches!(result, Value::Number(_)));
    }
}
