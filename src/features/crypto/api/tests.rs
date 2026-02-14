    use super::*;

    #[test]
    fn validate_proxy_url_accepts_supported_schemes() {
        assert!(validate_proxy_url("http://127.0.0.1:8080").is_ok());
        assert!(validate_proxy_url("https://proxy.example.com:443").is_ok());
        assert!(validate_proxy_url("socks5://127.0.0.1:9050").is_ok());
        assert!(validate_proxy_url("socks5h://127.0.0.1:9050").is_ok());
    }

    #[test]
    fn validate_proxy_url_rejects_invalid_values() {
        assert!(validate_proxy_url("").is_err());
        assert!(validate_proxy_url("ftp://127.0.0.1:21").is_err());
        assert!(validate_proxy_url("http://").is_err());
    }

    #[test]
    fn create_secure_client_rejects_invalid_proxy_config() {
        let bad_proxy = ProxyConfig {
            url: "ftp://127.0.0.1:21".to_string(),
        };
        let result = create_secure_client(Some(&bad_proxy));
        assert!(result.is_err());
    }

    #[test]
    fn create_secure_client_accepts_valid_proxy_config() {
        let proxy = ProxyConfig {
            url: "http://127.0.0.1:8080".to_string(),
        };
        let result = create_secure_client(Some(&proxy));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn fetch_crypto_prices_empty_list_returns_without_network() {
        let result = fetch_crypto_prices(Vec::new(), None).await;
        assert!(result.is_ok());
        assert!(result.expect("empty list should return ok").is_empty());
    }

    #[test]
    fn parse_mindicador_rate_uses_serie_value() {
        let body =
            br#"{"codigo":"dolar","serie":[{"fecha":"2026-02-10T03:00:00.000Z","valor":981.45}]}"#;
        let rate = parse_mindicador_rate(body).expect("mindicador sample must parse");
        assert_eq!(rate, 981.45);
    }

    #[test]
    fn parse_mindicador_rate_accepts_top_level_valor() {
        let body = br#"{"codigo":"dolar","valor":945.1}"#;
        let rate = parse_mindicador_rate(body).expect("top-level valor must parse");
        assert_eq!(rate, 945.1);
    }

    #[test]
    fn parse_usdt_fiat_rate_rejects_non_positive_value() {
        let body = br#"{"tether":{"clp":0.0}}"#;
        let err = parse_usdt_fiat_rate(body, "CLP").expect_err("non-positive rate must fail");
        assert_eq!(err, "Exchange rate out of expected range");
    }

    #[test]
    fn parse_currency_api_rate_reads_target_currency() {
        let body = br#"{"date":"2026-02-10","usd":{"eur":0.95,"gbp":0.79}}"#;
        let eur = parse_currency_api_rate(body, "EUR").expect("EUR rate must parse");
        let gbp = parse_currency_api_rate(body, "GBP").expect("GBP rate must parse");
        assert_eq!(eur, 0.95);
        assert_eq!(gbp, 0.79);
    }

    #[test]
    fn parse_usdt_fiat_rate_reads_requested_currency() {
        let body = br#"{"tether":{"eur":0.96,"mxn":17.20}}"#;
        let eur = parse_usdt_fiat_rate(body, "EUR").expect("EUR fallback rate must parse");
        let mxn = parse_usdt_fiat_rate(body, "MXN").expect("MXN fallback rate must parse");
        assert_eq!(eur, 0.96);
        assert_eq!(mxn, 17.20);
    }

    #[test]
    fn normalize_history_date_accepts_supported_formats() {
        let iso = normalize_history_date("2025-02-10").expect("ISO date must parse");
        let dmy = normalize_history_date("10-02-2025").expect("DMY date must parse");
        assert_eq!(iso, dmy);
        assert_eq!(iso.format("%Y-%m-%d").to_string(), "2025-02-10");
    }

    #[test]
    fn normalize_history_date_rejects_future_dates() {
        let err = normalize_history_date("2999-01-01").expect_err("future date must fail");
        assert_eq!(err, "Date cannot be in the future");
    }

    #[test]
    fn parse_coinpaprika_historical_price_reads_first_price() {
        let body = br#"[{"timestamp":"2025-02-10T00:00:00Z","price":43123.11}]"#;
        let price = parse_coinpaprika_historical_price(body).expect("CoinPaprika price must parse");
        assert_eq!(price, 43123.11);
    }

    #[test]
    fn parse_coinpaprika_historical_price_rejects_missing_price() {
        let body = br#"[{"timestamp":"2025-02-10T00:00:00Z"}]"#;
        let err = parse_coinpaprika_historical_price(body).expect_err("missing price must fail");
        assert_eq!(err, "CoinPaprika historical price not available");
    }

    #[test]
    fn parse_kraken_ohlc_close_price_reads_close_field() {
        let body = br#"{
            "error": [],
            "result": {
                "XXBTZUSD": [
                    [1739145600,"43000.1","44000.1","42000.1","43500.5","0","0","1"]
                ],
                "last": "1739232000"
            }
        }"#;
        let price = parse_kraken_ohlc_close_price(body).expect("Kraken close price must parse");
        assert_eq!(price, 43500.5);
    }

    #[test]
    fn parse_kraken_ohlc_close_price_rejects_api_error_payload() {
        let body = br#"{
            "error": ["EGeneral:Permission denied"],
            "result": {}
        }"#;
        let err = parse_kraken_ohlc_close_price(body).expect_err("kraken error payload must fail");
        assert_eq!(err, "Kraken returned historical data errors");
    }

    #[test]
    fn parse_kraken_ohlc_close_price_rejects_missing_candles() {
        let body = br#"{
            "error": [],
            "result": {
                "XXBTZUSD": [],
                "last": "1739232000"
            }
        }"#;
        let err = parse_kraken_ohlc_close_price(body).expect_err("missing candles must fail");
        assert_eq!(err, "Kraken historical close price not available");
    }

    #[test]
    fn kraken_pair_for_coingecko_id_maps_expected_assets() {
        assert_eq!(kraken_pair_for_coingecko_id("bitcoin"), Some("XBTUSD"));
        assert_eq!(kraken_pair_for_coingecko_id("monero"), Some("XMRUSD"));
        assert_eq!(kraken_pair_for_coingecko_id("unknown-coin"), None);
    }
