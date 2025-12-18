use std::collections::VecDeque;

pub struct VolatilityEngine {
    window_size: usize, // e.g., 60 for 60 seconds
    prices: VecDeque<f64>,
}

impl VolatilityEngine {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            prices: VecDeque::with_capacity(window_size),
        }
    }

    pub fn add_price(&mut self, price: f64) {
        if self.prices.len() >= self.window_size {
            self.prices.pop_front();
        }
        self.prices.push_back(price);
    }

    /// Calculate realized volatility as the standard deviation of log returns.
    pub fn calculate_realized_vol(&self) -> Option<f64> {
        if self.prices.len() < 2 {
            return None;
        }

        let mut returns = Vec::with_capacity(self.prices.len() - 1);
        for i in 1..self.prices.len() {
            let log_return = (self.prices[i] / self.prices[i - 1]).ln();
            returns.push(log_return);
        }

        let n = returns.len() as f64;
        let mean = returns.iter().sum::<f64>() / n;
        let variance = returns.iter().map(|&r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        
        // Annualize the volatility (assuming 1-second updates)
        // sqrt(variance * 60 * 60 * 24 * 365.25)
        let seconds_in_year = 31557600.0;
        Some((variance * seconds_in_year).sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volatility_calculation() {
        let mut engine = VolatilityEngine::new(10);
        // Minimal variance set
        for p in vec![100.0, 101.0, 100.0, 101.0, 100.0] {
            engine.add_price(p);
        }
        
        let vol = engine.calculate_realized_vol();
        assert!(vol.is_some());
        println!("Calculated Vol: {:?}", vol.unwrap());
    }
}
