use std::collections::BTreeMap;
use ordered_float::OrderedFloat;

#[derive(Debug, Default, Clone)]
pub struct OrderBook {
    pub bids: BTreeMap<OrderedFloat<f64>, f64>, // Price (desc) -> Size
    pub asks: BTreeMap<OrderedFloat<f64>, f64>, // Price (asc) -> Size
}

impl OrderBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_bid(&mut self, price: f64, size: f64) {
        if size == 0.0 {
            self.bids.remove(&OrderedFloat(price));
        } else {
            self.bids.insert(OrderedFloat(price), size);
        }
    }

    pub fn update_ask(&mut self, price: f64, size: f64) {
        if size == 0.0 {
            self.asks.remove(&OrderedFloat(price));
        } else {
            self.asks.insert(OrderedFloat(price), size);
        }
    }

    pub fn get_best_bid(&self) -> Option<(f64, f64)> {
        self.bids.iter().next_back().map(|(p, s)| (p.into_inner(), *s))
    }

    pub fn get_best_ask(&self) -> Option<(f64, f64)> {
        self.asks.iter().next().map(|(p, s)| (p.into_inner(), *s))
    }

    pub fn get_liquidity_at_price(&self, price: f64, is_bid: bool) -> f64 {
        let target = OrderedFloat(price);
        if is_bid {
            *self.bids.get(&target).unwrap_or(&0.0)
        } else {
            *self.asks.get(&target).unwrap_or(&0.0)
        }
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderbook_sorting() {
        let mut ob = OrderBook::new();
        ob.update_bid(100.0, 1.0);
        ob.update_bid(101.0, 2.0);
        ob.update_ask(105.0, 1.0);
        ob.update_ask(104.0, 2.0);

        assert_eq!(ob.get_best_bid(), Some((101.0, 2.0)));
        assert_eq!(ob.get_best_ask(), Some((104.0, 2.0)));
    }

    #[test]
    fn test_orderbook_removal() {
        let mut ob = OrderBook::new();
        ob.update_bid(100.0, 1.0);
        ob.update_bid(100.0, 0.0);
        assert!(ob.bids.is_empty());
    }
}
