use crate::types::{Asset, MarketMetadata};
use regex::Regex;

pub struct MarketParser {
    asset_regex: Regex,
    strike_regex: Regex,
}

impl MarketParser {
    pub fn new() -> Self {
        Self {
            asset_regex: Regex::new(r"(?i)(Bitcoin|BTC|Ethereum|ETH|Solana|SOL)").unwrap(),
            // Matches numbers like $98,123.45, 100k, 2500, etc.
            // Specifically looks for digits after indicators like >, <, or "above"
            strike_regex: Regex::new(r"(?:>|\babove\b|<)\s*\$?\s*([\d,]+(?:\.\d+)?)(k)?").unwrap(),
        }
    }

    pub fn parse(&self, title: &str) -> Option<(Asset, f64)> {
        let asset = self.detect_asset(title);
        if asset == Asset::Unknown {
            return None;
        }

        let strike = self.extract_strike(title)?;

        Some((asset, strike))
    }

    fn detect_asset(&self, title: &str) -> Asset {
        if let Some(mat) = self.asset_regex.find(title) {
            match mat.as_str().to_lowercase().as_str() {
                "bitcoin" | "btc" => Asset::BTC,
                "ethereum" | "eth" => Asset::ETH,
                "solana" | "sol" => Asset::SOL,
                _ => Asset::Unknown,
            }
        } else {
            Asset::Unknown
        }
    }

    fn extract_strike(&self, title: &str) -> Option<f64> {
        if let Some(caps) = self.strike_regex.captures(title) {
            let num_str = caps.get(1)?.as_str().replace(',', "");
            let mut val: f64 = num_str.parse().ok()?;
            
            // Handle 'k' suffix
            if caps.get(2).is_some() {
                val *= 1000.0;
            }
            
            Some(val)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bitcoin_complex() {
        let parser = MarketParser::new();
        let title = "Will Bitcoin be above $98,123.45 at 12:00?";
        let (asset, strike) = parser.parse(title).unwrap();
        assert_eq!(asset, Asset::BTC);
        assert_eq!(strike, 98123.45);
    }

    #[test]
    fn test_parse_eth_shorthand() {
        let parser = MarketParser::new();
        let title = "ETH > 2500 on Dec 17?";
        let (asset, strike) = parser.parse(title).unwrap();
        assert_eq!(asset, Asset::ETH);
        assert_eq!(strike, 2500.0);
    }

    #[test]
    fn test_parse_k_suffix() {
        let parser = MarketParser::new();
        let title = "Will BTC be above 100k?";
        let (asset, strike) = parser.parse(title).unwrap();
        assert_eq!(asset, Asset::BTC);
        assert_eq!(strike, 100000.0);
    }

    #[test]
    fn test_parse_failure() {
        let parser = MarketParser::new();
        let title = "Will it rain tomorrow?";
        assert!(parser.parse(title).is_none());
    }
}
