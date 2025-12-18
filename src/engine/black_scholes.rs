use statrs::distribution::{ContinuousCDF, Normal};
use std::f64::consts::E;

pub struct BlackScholes;

impl BlackScholes {
    /// Calculate the Fair Value of a Cash-or-Nothing binary option.
    /// C = e^(-r * T) * N(d2)
    /// where d2 = [ln(S/K) + (r - 0.5 * sigma^2) * T] / (sigma * sqrt(T))
    pub fn binary_call(
        spot: f64,
        strike: f64,
        time_to_expiry_years: f64,
        rate: f64,
        volatility: f64,
    ) -> f64 {
        if time_to_expiry_years <= 0.0 {
            return if spot >= strike { 1.0 } else { 0.0 };
        }

        let sigma_sqrt_t = volatility * time_to_expiry_years.sqrt();
        let d2 = ( (spot / strike).ln() + (rate - 0.5 * volatility.powi(2)) * time_to_expiry_years ) / sigma_sqrt_t;

        let n = Normal::new(0.0, 1.0).unwrap();
        let n_d2 = n.cdf(d2);

        E.powf(-rate * time_to_expiry_years) * n_d2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_call_atm_short_expiry() {
        // ATM Option: Spot = Strike = 100.0
        // Short expiry: 1 minute = 1.0 / (365.25 * 24.0 * 60.0)
        let spot = 100.0;
        let strike = 100.0;
        let t = 1.0 / 525600.0; // 1 minute in years
        let r = 0.05; // 5% risk-free rate
        let sigma = 0.30; // 30% IV

        let price = BlackScholes::binary_call(spot, strike, t, r, sigma);
        
        // ATM binary call should be approx 0.50 (slightly above due to rate)
        println!("ATM Price: {}", price);
        assert!(price > 0.49 && price < 0.51);
    }

    #[test]
    fn test_binary_call_deep_itm() {
        let price = BlackScholes::binary_call(110.0, 100.0, 0.1, 0.05, 0.2);
        assert!(price > 0.9);
    }

    #[test]
    fn test_binary_call_deep_otm() {
        let price = BlackScholes::binary_call(90.0, 100.0, 0.1, 0.05, 0.2);
        assert!(price < 0.1);
    }
}
