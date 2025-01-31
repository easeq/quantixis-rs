use crate::ast::{Executor, Value};
use crate::extract_args;
use quantixis_macros::quantinxis_fn;

pub fn register(executor: &mut Executor) {
    executor.register_function("rate_of_change", rate_of_change);
    executor.register_function("stochastic", stochastic);
    executor.register_function("momentum", momentum);
    executor.register_function("commodity_channel_index", commodity_channel_index);
    // executor.register_function("chande_momentum_oscillator", chande_momentum_oscillator);
    executor.register_function("relative_vigor_index", relative_vigor_index);
    executor.register_function("williams_percent_r", williams_percent_r);
    executor.register_function("awesome_osc", awesome_oscillator);
    executor.register_function("ad_oscillator", ad_oscillator);
    executor.register_function("klinger_oscillator", klinger_oscillator);
    executor.register_function("choppiness_index", choppiness_index);
}

#[quantinxis_fn]
fn rate_of_change(prices: Vec<f64>, period: f64) -> Result<Value, String> {
    let period = period as usize;
    if prices.len() <= period {
        return Err("Not enough data points to compute ROC".to_string());
    }

    let roc = ((prices.last().unwrap() - prices[prices.len() - period - 1])
        / prices[prices.len() - period - 1])
        * 100.0;
    Ok(Value::Number(roc))
}

#[quantinxis_fn]
fn stochastic(highs: Vec<f64>, lows: Vec<f64>, closes: Vec<f64>) -> Result<Value, String> {
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

#[quantinxis_fn]
fn momentum(prices: Vec<f64>, period: f64) -> Result<Value, String> {
    let period = period as usize;
    if prices.len() <= period {
        return Err("Not enough data points to compute Momentum".to_string());
    }

    let momentum = prices.last().unwrap() - prices[prices.len() - period - 1];
    Ok(Value::Number(momentum))
}

/// Commodity Channel Index (CCI)
/// Formula: CCI = (Typical Price - SMA) / (0.015 * Mean Deviation)
#[quantinxis_fn()]
fn commodity_channel_index(
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    period: f64,
) -> Result<Value, String> {
    let period = period as usize;
    if high.len() < period || low.len() < period || close.len() < period {
        return Err("Not enough data points to compute CCI".to_string());
    }

    let typical_prices: Vec<f64> = high
        .iter()
        .zip(low.iter())
        .zip(close.iter())
        .map(|((&h, &l), &c)| (h + l + c) / 3.0)
        .collect();

    let sma: f64 = typical_prices.iter().rev().take(period).sum::<f64>() / period as f64;

    let mean_deviation: f64 = typical_prices
        .iter()
        .rev()
        .take(period)
        .map(|&tp| (tp - sma).abs())
        .sum::<f64>()
        / period as f64;

    let cci = (typical_prices.last().unwrap() - sma) / (0.015 * mean_deviation);

    Ok(Value::Number(cci))
}

/// Chande Momentum Oscillator (CMO)
/// Formula: CMO = (Sum of Up Changes - Sum of Down Changes) / (Sum of Up Changes + Sum of Down Changes) * 100
#[quantinxis_fn]
fn chande_momentum_oscillator(prices: Vec<f64>, period: f64) -> Result<Value, String> {
    let period = period as usize;
    if prices.len() < period + 1 {
        return Err("Not enough data points to compute CMO".to_string());
    }

    let mut up_sum = 0.0;
    let mut down_sum = 0.0;

    for i in 1..=period {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            up_sum += change;
        } else {
            down_sum -= change; // Down changes are positive
        }
    }

    if (up_sum + down_sum) == 0.0 {
        return Ok(Value::Number(0.0));
    }

    let cmo = ((up_sum - down_sum) / (up_sum + down_sum)) * 100.0;
    Ok(Value::Number(cmo))
}

/// Awesome Oscillator (AO)
/// Formula: AO = 5-period SMA of Median Price - 34-period SMA of Median Price
#[quantinxis_fn]
fn awesome_oscillator(high: Vec<f64>, low: Vec<f64>) -> Result<Value, String> {
    if high.len() < 34 || low.len() < 34 {
        return Err("Not enough data points to compute AO".to_string());
    }

    let median_prices: Vec<f64> = high
        .iter()
        .zip(low.iter())
        .map(|(&h, &l)| (h + l) / 2.0)
        .collect();

    let sma5 = median_prices.iter().rev().take(5).sum::<f64>() / 5.0;
    let sma34 = median_prices.iter().rev().take(34).sum::<f64>() / 34.0;

    let ao = sma5 - sma34;
    Ok(Value::Number(ao))
}

#[quantinxis_fn]
fn relative_vigor_index(closes: Vec<f64>, opens: Vec<f64>) -> Result<Value, String> {
    if closes.len() != opens.len() || closes.is_empty() {
        return Err(
            "Closing and opening price arrays must have the same nonzero length".to_string(),
        );
    }

    let sum_close_open: f64 = closes.iter().zip(opens.iter()).map(|(c, o)| c - o).sum();
    let sum_high_low: f64 = closes.iter().zip(opens).map(|(c, o)| c + o).sum();

    let rvi = sum_close_open / sum_high_low;
    Ok(Value::Number(rvi))
}

#[quantinxis_fn]
fn williams_percent_r(highs: Vec<f64>, lows: Vec<f64>, closes: Vec<f64>) -> Result<Value, String> {
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

#[quantinxis_fn]
fn ad_oscillator(
    highs: Vec<f64>,
    lows: Vec<f64>,
    closes: Vec<f64>,
    volumes: Vec<f64>,
) -> Result<Value, String> {
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

#[quantinxis_fn]
fn klinger_oscillator(
    highs: Vec<f64>,
    lows: Vec<f64>,
    closes: Vec<f64>,
    volumes: Vec<f64>,
) -> Result<Value, String> {
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

#[quantinxis_fn]
fn choppiness_index(highs: Vec<f64>, lows: Vec<f64>, closes: Vec<f64>) -> Result<Value, String> {
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

    // #[test]
    // fn test_awesome_oscillator() {
    //     let result = awesome_oscillator(&[Value::Array(vec![
    //         1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0,
    //     ])])
    //     .unwrap();
    //     assert!(matches!(result, Value::Number(_)));
    // }

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
