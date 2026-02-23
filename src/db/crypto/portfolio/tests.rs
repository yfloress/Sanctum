// Sanctum — a privacy-first personal finance, crypto, and habits vault.
// Copyright (C) 2026  Kyronix
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

    use super::*;

    fn tx(
        id: &str,
        coin_id: &str,
        symbol: &str,
        tx_type: &str,
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
            tx_type.to_string(),
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

    #[test]
    fn aggregate_fee_coin_uses_catalog_symbol_when_created_from_fee_first() {
        let mut btc_buy_with_usdt_fee = tx(
            "buy-btc",
            "bitcoin",
            "BTC",
            "trade",
            Some("buy"),
            0.1,
            Some(50_000.0),
            "2024-01-01",
            None,
        );
        btc_buy_with_usdt_fee.fee_coin_id = Some("tether".to_string());
        btc_buy_with_usdt_fee.fee_amount = Some(1.0);

        let usdt_deposit = tx(
            "dep-usdt",
            "tether",
            "USDT",
            "transfer",
            Some("deposit"),
            10.0,
            None,
            "2024-01-01",
            None,
        );

        let assets =
            Database::aggregate_crypto_transactions(vec![btc_buy_with_usdt_fee, usdt_deposit]);

        let usdt = assets
            .iter()
            .find(|a| a.coin_id == "tether")
            .expect("expected USDT holding");
        assert_eq!(usdt.symbol, "USDT");
        assert!((usdt.total_amount - 10.0).abs() < 0.00000001);
    }

    #[test]
    fn aggregate_overrides_legacy_tether_symbol_with_canonical_usdt() {
        let legacy_tether_tx = tx(
            "legacy-usdt",
            "tether",
            "TETHER",
            "transfer",
            Some("deposit"),
            2.5,
            None,
            "2024-01-03",
            None,
        );

        let assets = Database::aggregate_crypto_transactions(vec![legacy_tether_tx]);
        let usdt = assets
            .iter()
            .find(|a| a.coin_id == "tether")
            .expect("expected USDT holding");
        assert_eq!(usdt.symbol, "USDT");
        assert!((usdt.total_amount - 2.5).abs() < 0.00000001);
    }

    #[test]
    fn aggregate_keeps_real_dust_balances_visible() {
        let dust_tx = tx(
            "dust-usdt",
            "tether",
            "USDT",
            "transfer",
            Some("deposit"),
            0.0000064,
            None,
            "2024-01-04",
            None,
        );

        let assets = Database::aggregate_crypto_transactions(vec![dust_tx]);
        assert_eq!(assets.len(), 1);
        let usdt = assets
            .iter()
            .find(|a| a.coin_id == "tether")
            .expect("expected dust USDT holding");
        assert!((usdt.total_amount - 0.0000064).abs() < 0.000000000001);
    }

    #[test]
    fn aggregate_filters_near_zero_floating_residue() {
        let residue_tx = tx(
            "residue-usdt",
            "tether",
            "USDT",
            "transfer",
            Some("deposit"),
            0.0000000000001,
            None,
            "2024-01-04",
            None,
        );

        let assets = Database::aggregate_crypto_transactions(vec![residue_tx]);
        assert!(assets.is_empty());
    }
