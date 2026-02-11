    use super::*;

    fn tx(
        id: &str,
        coin_id: &str,
        symbol: &str,
        fiscal_type: &str,
        subtype: Option<&str>,
        amount: f64,
        price_per_coin: Option<f64>,
        date: &str,
        related_tx_id: Option<&str>,
    ) -> CryptoTransaction {
        let mut tx = CryptoTransaction::new(
            id.to_string(),
            "wallet-1".to_string(),
            coin_id.to_string(),
            symbol.to_string(),
            fiscal_type.to_string(),
            amount,
            price_per_coin,
            None,
            date.to_string(),
            None,
        );
        tx.subtype = subtype.map(str::to_string);
        tx.related_tx_id = related_tx_id.map(str::to_string);
        tx
    }

    #[test]
    fn aggregate_swap_pair_handles_trade_swap_both_sides() {
        let buy_btc = tx(
            "buy-1",
            "bitcoin",
            "BTC",
            "trade",
            Some("buy"),
            1.0,
            Some(100.0),
            "2024-01-01",
            None,
        );

        let swap_out_btc = tx(
            "swap-out",
            "bitcoin",
            "BTC",
            "trade",
            Some("swap"),
            1.0,
            None,
            "2024-01-02",
            Some("swap-in"),
        );

        let swap_in_eth = tx(
            "swap-in",
            "ethereum",
            "ETH",
            "trade",
            Some("swap"),
            10.0,
            None,
            "2024-01-02",
            Some("swap-out"),
        );

        let assets = Database::aggregate_crypto_transactions(vec![buy_btc, swap_out_btc, swap_in_eth]);

        assert_eq!(assets.len(), 1);
        let eth = assets
            .iter()
            .find(|a| a.coin_id == "ethereum")
            .expect("expected ETH holding after swap");
        assert!((eth.total_amount - 10.0).abs() < 0.00000001);
    }

    #[test]
    fn aggregate_swap_pair_prefers_side_with_available_balance_over_id_order() {
        // IDs intentionally ordered so lexical fallback would choose swap-in as source.
        let buy_btc = tx(
            "buy-1",
            "bitcoin",
            "BTC",
            "trade",
            Some("buy"),
            1.0,
            Some(100.0),
            "2024-01-01",
            None,
        );
        let swap_out_btc = tx(
            "z-swap-out",
            "bitcoin",
            "BTC",
            "trade",
            Some("swap"),
            1.0,
            None,
            "2024-01-02",
            Some("a-swap-in"),
        );
        let swap_in_eth = tx(
            "a-swap-in",
            "ethereum",
            "ETH",
            "trade",
            Some("swap"),
            10.0,
            None,
            "2024-01-02",
            Some("z-swap-out"),
        );

        let assets = Database::aggregate_crypto_transactions(vec![buy_btc, swap_out_btc, swap_in_eth]);
        let eth = assets
            .iter()
            .find(|a| a.coin_id == "ethereum")
            .expect("expected ETH holding after swap");
        assert!((eth.total_amount - 10.0).abs() < 0.00000001);
    }
