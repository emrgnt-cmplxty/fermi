
extern crate engine;
extern crate proc;

pub use engine::domain::OrderSide;
pub use engine::orderbook::{Orderbook, OrderProcessingResult, Success, Failed};
pub use engine::orders;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Eq, Debug, Copy, Clone)]
    enum Asset {
        USD,
        EUR,
        BTC,
        ETH,
    }

    fn parse_asset(asset: &str) -> Option<Asset> {
        match asset {
            "USD" => Some(Asset::USD),
            "EUR" => Some(Asset::EUR),
            "BTC" => Some(Asset::BTC),
            "ETH" => Some(Asset::ETH),
            _ => None,
        }
    }

    #[test]
    fn market_order_on_empty_orderbook() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let base_asset = parse_asset("BTC").unwrap();
        let quote_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_market_order_request(
            base_asset,
            quote_asset,
            OrderSide::Bid,
            2,
            SystemTime::now(),
        );

        // process market order
        let res = orderbook.process_order(order1);

        if !match res[0] {
            Ok(Success::Accepted { order_id: 1, .. }) => true,
            _ => false,
        } ||
            !match res[1] {
                Err(Failed::NoMatch(1)) => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res)
        }
    }


    #[test]
    fn market_order_partial_match() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let base_asset = parse_asset("BTC").unwrap();
        let quote_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Bid,
            10,
            2,
            SystemTime::now(),
        );

        let order2 = orders::new_market_order_request(
            base_asset,
            quote_asset,
            OrderSide::Ask,
            1,
            SystemTime::now(),
        );

        orderbook.process_order(order1);
        let res = orderbook.process_order(order2);

        if !match res[0] {
            Ok(Success::Accepted { order_id: 2, .. }) => true,
            _ => false,
        } ||
            !match res[1] {
                Ok(Success::Filled {
                       order_id: 2,
                       price,
                       qty,
                       ..
                   }) if price == 10 && qty == 1 => true,
                _ => false,
            } ||
            !match res[2] {
                Ok(Success::PartiallyFilled {
                       order_id: 1,
                       price,
                       qty,
                       ..
                   }) if price == 10 && qty == 1 => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res)
        }
    }


    #[test]
    fn market_order_two_orders_match() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let base_asset = parse_asset("BTC").unwrap();
        let quote_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Bid,
            10,
            10,
            SystemTime::now(),
        );

        let order2 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Bid,
            12,
            10,
            SystemTime::now(),
        );

        let order3 = orders::new_market_order_request(
            base_asset,
            quote_asset,
            OrderSide::Ask,
            15,
            SystemTime::now(),
        );

        orderbook.process_order(order1);
        orderbook.process_order(order2);
        let res = orderbook.process_order(order3);

        if !match res[0] {
            Ok(Success::Accepted { order_id: 3, .. }) => true,
            _ => false,
        } ||
            !match res[1] {
                Ok(Success::PartiallyFilled {
                       order_id: 3,
                       price,
                       qty,
                       ..
                   }) if price == 12 && qty == 10 => true,
                _ => false,
            } ||
            !match res[2] {
                Ok(Success::Filled {
                       order_id: 2,
                       price,
                       qty,
                       ..
                   }) if price == 12 && qty == 10 => true,
                _ => false,
            } ||
            !match res[3] {
                Ok(Success::Filled {
                       order_id: 3,
                       price,
                       qty,
                       ..
                   }) if price == 10 && qty == 5 => true,
                _ => false,
            } ||
            !match res[4] {
                Ok(Success::PartiallyFilled {
                       order_id: 1,
                       price,
                       qty,
                       ..
                   }) if price == 10 && qty == 5 => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res)
        }
    }


    #[test]
    fn limit_order_on_empty_orderbook() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let base_asset = parse_asset("BTC").unwrap();
        let quote_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Bid,
            100,
            20,
            SystemTime::now(),
        );

        // process order
        let res = orderbook.process_order(order1);

        if !match res[0] {
            Ok(Success::Accepted { order_id: 1, .. }) => true,
            _ => false,
        }
        {
            panic!("unexpected event sequence: {:?}", res)
        }
    }


    #[test]
    fn limit_order_partial_match() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let base_asset = parse_asset("BTC").unwrap();
        let quote_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Bid,
            100,
            100,
            SystemTime::now(),
        );

        let order2 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Ask,
            90,
            50,
            SystemTime::now(),
        );

        orderbook.process_order(order1);
        let res = orderbook.process_order(order2);

        if !match res[0] {
            Ok(Success::Accepted { order_id: 2, .. }) => true,
            _ => false,
        } ||
            !match res[1] {
                Ok(Success::Filled {
                       order_id: 2,
                       price,
                       qty,
                       ..
                   }) if price == 100 && qty == 50 => true,
                _ => false,
            } ||
            !match res[2] {
                Ok(Success::PartiallyFilled {
                       order_id: 1,
                       price,
                       qty,
                       ..
                   }) if price == 100 && qty == 50 => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res)
        }
    }


    #[test]
    fn limit_order_exact_match() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let base_asset = parse_asset("BTC").unwrap();
        let quote_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Bid,
            100,
            10,
            SystemTime::now(),
        );

        let order2 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Ask,
            90,
            5,
            SystemTime::now(),
        );

        orderbook.process_order(order1);
        let res = orderbook.process_order(order2);

        if !match res[0] {
            Ok(Success::Accepted { order_id: 2, .. }) => true,
            _ => false,
        } ||
            !match res[1] {
                Ok(Success::Filled {
                       order_id: 2,
                       price,
                       qty,
                       ..
                   }) if price == 100 && qty == 5 => true,
                _ => false,
            } ||
            !match res[2] {
                Ok(Success::PartiallyFilled {
                       order_id: 1,
                       price,
                       qty,
                       ..
                   }) if price == 100 && qty == 5 => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res)
        }

        let order3 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Ask,
            80,
            5,
            SystemTime::now(),
        );

        let res2 = orderbook.process_order(order3);

        if !match res2[0] {
            Ok(Success::Accepted { order_id: 3, .. }) => true,
            _ => false,
        } ||
            !match res2[1] {
                Ok(Success::Filled {
                       order_id: 3,
                       price,
                       qty,
                       ..
                   }) if price == 100 && qty == 5 => true,
                _ => false,
            } ||
            !match res2[2] {
                Ok(Success::Filled {
                       order_id: 1,
                       price,
                       qty,
                       ..
                   }) if price == 100 && qty == 5 => true,
                _ => false,
            }
        {
            panic!("unexpected event sequence: {:?}", res2)
        }

        assert_eq!(orderbook.current_spread(), None);
    }


    #[test]
    fn current_spread() {
        use std::time::SystemTime;

        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let base_asset = parse_asset("BTC").unwrap();
        let quote_asset = parse_asset("USD").unwrap();

        let order1 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Bid,
            100,
            10,
            SystemTime::now(),
        );

        // not enough orders to calculate
        assert_eq!(orderbook.current_spread(), None);

        let order2 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Ask,
            120,
            5,
            SystemTime::now(),
        );

        let order3 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Ask,
            125,
            25,
            SystemTime::now(),
        );

        orderbook.process_order(order1);
        orderbook.process_order(order2);
        orderbook.process_order(order3);

        assert_eq!(orderbook.current_spread(), Some((100, 120)));

        // wider spread
        let order4 = orders::new_limit_order_request(
            base_asset,
            quote_asset,
            OrderSide::Bid,
            140,
            15,
            SystemTime::now(),
        );
        let res = orderbook.process_order(order4);
        println!("{:?}", res);

        assert_eq!(orderbook.current_spread(), Some((100, 125)));
    }
}
