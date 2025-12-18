use crate::types::{Asset, MarketMetadata};
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref ASSET_REGEX: Regex = Regex::new(r"(?i)(Bitcoin|BTC|Ethereum|ETH|Solana|SOL)").unwrap();
    // Matches numbers like $98,123.45, 100k, 2500, etc. after keywords
    static ref STRIKE_REGEX: Regex = Regex::new(r"(?:>|\babove\b|\bbelow\b|<)\s*\$?\s*([\d,]+(?:\.\d+)?)(k)?").unwrap();
}

pub struct MarketParser;

impl MarketParser {
    pub fn new() -> Self {
        Self
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
        if let Some(mat) = ASSET_REGEX.find(title) {
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
        if let Some(caps) = STRIKE_REGEX.captures(title) {
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
        let title = "Will Bitcoin be above $98,123.45 at 12:00 UTC?";
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
    fn test_parse_below() {
        let parser = MarketParser::new();
        let title = "Will Solana be below $145.50?";
        let (asset, strike) = parser.parse(title).unwrap();
        assert_eq!(asset, Asset::SOL);
        assert_eq!(strike, 145.50);
    }
}
