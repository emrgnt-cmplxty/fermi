#[cfg(test)]
mod tests {

    use gdex_engine::{order_book::Orderbook, orders};
    use gdex_types::order_book::{Failed, OrderSide, Success};
    use std::time::SystemTime;

    const BASE_ASSET_ID: u64 = 0;
    const QUOTE_ASSET_ID: u64 = 1;

    #[test]
    fn market_order_on_empty_orderbook() {
        let mut orderbook = Orderbook::new(BASE_ASSET_ID, QUOTE_ASSET_ID);

        let order1 =
            orders::create_market_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, OrderSide::Bid, 2, SystemTime::now());

        // process market order
        let res = orderbook.process_order(order1);

        if !match res[0] {
            Ok(Success::Accepted { order_id: 1, .. }) => true,
            _ => false,
        } || !match res[1] {
            Err(Failed::NoMatch(1)) => true,
            _ => false,
        } {
            panic!("unexpected event sequence: {:?}", res)
        }
    }

    #[test]
    fn market_order_partial_match() {
        let mut orderbook = Orderbook::new(BASE_ASSET_ID, QUOTE_ASSET_ID);

        let order1 =
            orders::create_limit_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, OrderSide::Bid, 10, 2, SystemTime::now());

        let order2 =
            orders::create_market_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, OrderSide::Ask, 1, SystemTime::now());

        orderbook.process_order(order1);
        let res = orderbook.process_order(order2);

        if !match res[0] {
            Ok(Success::Accepted { order_id: 2, .. }) => true,
            _ => false,
        } || !match res[1] {
            Ok(Success::Filled {
                order_id: 2,
                price,
                quantity,
                ..
            }) if price == 10 && quantity == 1 => true,
            _ => false,
        } || !match res[2] {
            Ok(Success::PartiallyFilled {
                order_id: 1,
                price,
                quantity,
                ..
            }) if price == 10 && quantity == 1 => true,
            _ => false,
        } {
            panic!("unexpected event sequence: {:?}", res)
        }
    }

    #[test]
    fn market_order_two_orders_match() {
        let mut orderbook = Orderbook::new(BASE_ASSET_ID, QUOTE_ASSET_ID);

        let order1 =
            orders::create_limit_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, OrderSide::Bid, 10, 10, SystemTime::now());

        let order2 =
            orders::create_limit_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, OrderSide::Bid, 12, 10, SystemTime::now());

        let order3 =
            orders::create_market_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, OrderSide::Ask, 15, SystemTime::now());

        orderbook.process_order(order1);
        orderbook.process_order(order2);
        let res = orderbook.process_order(order3);

        if !match res[0] {
            Ok(Success::Accepted { order_id: 3, .. }) => true,
            _ => false,
        } || !match res[1] {
            Ok(Success::PartiallyFilled {
                order_id: 3,
                price,
                quantity,
                ..
            }) if price == 12 && quantity == 10 => true,
            _ => false,
        } || !match res[2] {
            Ok(Success::Filled {
                order_id: 2,
                price,
                quantity,
                ..
            }) if price == 12 && quantity == 10 => true,
            _ => false,
        } || !match res[3] {
            Ok(Success::Filled {
                order_id: 3,
                price,
                quantity,
                ..
            }) if price == 10 && quantity == 5 => true,
            _ => false,
        } || !match res[4] {
            Ok(Success::PartiallyFilled {
                order_id: 1,
                price,
                quantity,
                ..
            }) if price == 10 && quantity == 5 => true,
            _ => false,
        } {
            panic!("unexpected event sequence: {:?}", res)
        }
    }

    #[test]
    fn limit_order_on_empty_orderbook() {
        let mut orderbook = Orderbook::new(BASE_ASSET_ID, QUOTE_ASSET_ID);

        let order1 = orders::create_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
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
        } {
            panic!("unexpected event sequence: {:?}", res)
        }
    }

    #[test]
    fn limit_order_partial_match() {
        let mut orderbook = Orderbook::new(BASE_ASSET_ID, QUOTE_ASSET_ID);

        let order1 = orders::create_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid,
            100,
            100,
            SystemTime::now(),
        );

        let order2 =
            orders::create_limit_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, OrderSide::Ask, 90, 50, SystemTime::now());

        orderbook.process_order(order1);
        let res = orderbook.process_order(order2);

        if !match res[0] {
            Ok(Success::Accepted { order_id: 2, .. }) => true,
            _ => false,
        } || !match res[1] {
            Ok(Success::Filled {
                order_id: 2,
                price,
                quantity,
                ..
            }) if price == 100 && quantity == 50 => true,
            _ => false,
        } || !match res[2] {
            Ok(Success::PartiallyFilled {
                order_id: 1,
                price,
                quantity,
                ..
            }) if price == 100 && quantity == 50 => true,
            _ => false,
        } {
            panic!("unexpected event sequence: {:?}", res)
        }
    }

    #[test]
    fn limit_order_exact_match() {
        let mut orderbook = Orderbook::new(BASE_ASSET_ID, QUOTE_ASSET_ID);

        let order1 = orders::create_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid,
            100,
            10,
            SystemTime::now(),
        );

        let order2 =
            orders::create_limit_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, OrderSide::Ask, 90, 5, SystemTime::now());

        orderbook.process_order(order1);
        let res = orderbook.process_order(order2);

        if !match res[0] {
            Ok(Success::Accepted { order_id: 2, .. }) => true,
            _ => false,
        } || !match res[1] {
            Ok(Success::Filled {
                order_id: 2,
                price,
                quantity,
                ..
            }) if price == 100 && quantity == 5 => true,
            _ => false,
        } || !match res[2] {
            Ok(Success::PartiallyFilled {
                order_id: 1,
                price,
                quantity,
                ..
            }) if price == 100 && quantity == 5 => true,
            _ => false,
        } {
            panic!("unexpected event sequence: {:?}", res)
        }

        let order3 =
            orders::create_limit_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, OrderSide::Ask, 80, 5, SystemTime::now());

        let res2 = orderbook.process_order(order3);

        if !match res2[0] {
            Ok(Success::Accepted { order_id: 3, .. }) => true,
            _ => false,
        } || !match res2[1] {
            Ok(Success::Filled {
                order_id: 3,
                price,
                quantity,
                ..
            }) if price == 100 && quantity == 5 => true,
            _ => false,
        } || !match res2[2] {
            Ok(Success::Filled {
                order_id: 1,
                price,
                quantity,
                ..
            }) if price == 100 && quantity == 5 => true,
            _ => false,
        } {
            panic!("unexpected event sequence: {:?}", res2)
        }

        assert_eq!(orderbook.current_spread(), None);
    }

    #[test]
    fn current_spread() {
        let mut orderbook = Orderbook::new(BASE_ASSET_ID, QUOTE_ASSET_ID);

        let order1 = orders::create_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid,
            100,
            10,
            SystemTime::now(),
        );

        // not enough orders to calculate
        assert_eq!(orderbook.current_spread(), None);

        let order2 =
            orders::create_limit_order_request(BASE_ASSET_ID, QUOTE_ASSET_ID, OrderSide::Ask, 120, 5, SystemTime::now());

        let order3 = orders::create_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
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
        let order4 = orders::create_limit_order_request(
            BASE_ASSET_ID,
            QUOTE_ASSET_ID,
            OrderSide::Bid,
            140,
            15,
            SystemTime::now(),
        );
        orderbook.process_order(order4);

        assert_eq!(orderbook.current_spread(), Some((100, 125)));
    }
}
