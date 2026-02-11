    use super::*;
    use crate::db::Database;
    use crate::features::crypto::TaxReportSummary;
    use secrecy::SecretString;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    struct TestServiceHarness {
        service: CryptoService,
        test_dir: PathBuf,
    }

    impl Drop for TestServiceHarness {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.test_dir);
        }
    }

    fn new_test_service() -> TestServiceHarness {
        let base_dir = std::env::temp_dir().join(format!("sanctum-tax-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&base_dir).expect("create test dir");
        let db_path = base_dir.join("vault.db");
        let password = SecretString::from("test-password-123".to_string());
        let db = Database::init(db_path, &password).expect("init test database");
        let service = CryptoService::new(Arc::new(Mutex::new(Some(db))));
        TestServiceHarness {
            service,
            test_dir: base_dir,
        }
    }

    fn tx(
        id: &str,
        tx_type: &str,
        subtype: Option<&str>,
        amount: f64,
        price: Option<f64>,
        date: &str,
    ) -> CryptoTransaction {
        let mut tx = CryptoTransaction::new(
            id.to_string(),
            "wallet-1".to_string(),
            "bitcoin".to_string(),
            "BTC".to_string(),
            tx_type.to_string(),
            amount,
            price,
            None,
            date.to_string(),
            None,
        );
        tx.subtype = subtype.map(str::to_string);
        tx
    }

    #[test]
    fn compute_taxable_income_uses_override_proceeds_priority() {
        let period = parse_period("2024").expect("valid period");

        let income_with_price = tx("i1", "income", Some("airdrop"), 1.0, Some(100.0), "2024-03-10");
        let mut income_with_override =
            tx("i2", "income", Some("reward"), 2.0, Some(200.0), "2024-05-10");
        income_with_override.override_proceeds = Some(50.0);
        let trade = tx("t1", "trade", Some("buy"), 1.0, Some(999.0), "2024-06-10");
        let outside_period = tx("i3", "income", Some("staking"), 1.0, Some(10.0), "2023-12-31");

        let (total, count, warnings) = compute_taxable_income(
            &[income_with_price, income_with_override, trade, outside_period],
            &period,
        );

        assert!((total - 150.0).abs() < 0.0001);
        assert_eq!(count, 2);
        assert!(warnings.is_empty());
    }

    #[test]
    fn compute_taxable_income_warns_when_price_missing() {
        let period = parse_period("2024").expect("valid period");
        let income_missing_price = tx("i1", "income", Some("gift"), 1.0, None, "2024-01-10");

        let (total, count, warnings) = compute_taxable_income(&[income_missing_price], &period);
        assert_eq!(total, 0.0);
        assert_eq!(count, 1);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "income_missing_price");
        assert_eq!(warnings[0].tx_id.as_deref(), Some("i1"));
    }

    #[test]
    fn build_readiness_sets_prices_error_on_invalid_warning() {
        let report = TaxReport {
            period_id: "2024".to_string(),
            period_start: "2024-01-01".to_string(),
            period_end: "2024-12-31".to_string(),
            jurisdiction: "chile".to_string(),
            method: "fifo".to_string(),
            summary: TaxReportSummary::default(),
            disposals: Vec::new(),
            warnings: vec![crate::features::crypto::TaxWarning {
                code: "invalid_date".to_string(),
                message: "bad date".to_string(),
                tx_id: None,
            }],
        };

        let readiness = build_readiness(&report, 3, 0, 0, TaxJurisdiction::Chile);
        let prices = readiness
            .iter()
            .find(|r| r.code == "prices")
            .expect("prices readiness item");
        assert_eq!(prices.status, "error");
        assert_eq!(prices.detail, "invalid");
    }

    #[test]
    fn history_csv_includes_fiscal_type_subtype_and_mechanical_type() {
        let period = parse_period("2024").expect("valid period");
        let tx = tx("s1", "trade", Some("swap"), 0.1, Some(50000.0), "2024-01-10");

        let csv = build_transaction_history_csv(&[tx], &period, "USD", 0.0);
        assert!(csv.contains("type,subtype,mechanical_type"));
        assert!(csv.contains(",trade,swap,swap,"));
    }

    #[test]
    fn history_csv_converts_fiat_values_for_clp_export() {
        let period = parse_period("2024").expect("valid period");
        let tx = tx("s1", "trade", Some("sell"), 0.1, Some(10.0), "2024-01-10");

        let csv = build_transaction_history_csv(&[tx], &period, "CLP", 1000.0);
        assert!(csv.contains("fiat_currency"));
        assert!(csv.contains(",10000.00000000,"));
        assert!(csv.contains(",CLP,"));
    }

    #[test]
    fn generate_tax_report_excludes_wallet_ids_from_settings() {
        let harness = new_test_service();
        let service = &harness.service;

        let wallet_excluded = service
            .add_wallet("Excluded wallet".to_string(), "exchange".to_string(), None)
            .expect("create excluded wallet");
        let wallet_included = service
            .add_wallet("Included wallet".to_string(), "exchange".to_string(), None)
            .expect("create included wallet");

        service
            .add_crypto_transaction(
                wallet_excluded.clone(),
                "bitcoin".to_string(),
                "BTC".to_string(),
                "trade".to_string(),
                1.0,
                Some(100.0),
                None,
                None,
                None,
                "2024-01-10".to_string(),
                None,
                Some("buy".to_string()),
                None,
                None,
            )
            .expect("buy in excluded wallet");
        let excluded_sell_id = service
            .add_crypto_transaction(
                wallet_excluded.clone(),
                "bitcoin".to_string(),
                "BTC".to_string(),
                "trade".to_string(),
                1.0,
                Some(150.0),
                None,
                None,
                None,
                "2024-02-10".to_string(),
                None,
                Some("sell".to_string()),
                None,
                None,
            )
            .expect("sell in excluded wallet");

        service
            .add_crypto_transaction(
                wallet_included.clone(),
                "bitcoin".to_string(),
                "BTC".to_string(),
                "trade".to_string(),
                1.0,
                Some(200.0),
                None,
                None,
                None,
                "2024-03-10".to_string(),
                None,
                Some("buy".to_string()),
                None,
                None,
            )
            .expect("buy in included wallet");
        let included_sell_id = service
            .add_crypto_transaction(
                wallet_included,
                "bitcoin".to_string(),
                "BTC".to_string(),
                "trade".to_string(),
                1.0,
                Some(260.0),
                None,
                None,
                None,
                "2024-04-10".to_string(),
                None,
                Some("sell".to_string()),
                None,
                None,
            )
            .expect("sell in included wallet");

        let mut settings = TaxPeriodSettings::defaults_for("2024");
        settings.jurisdiction = TaxJurisdiction::Usa;
        settings.excluded_wallet_ids = vec![wallet_excluded];

        service
            .save_tax_settings(settings)
            .expect("save tax settings with exclusion");

        let report = service
            .generate_tax_report("2024".to_string())
            .expect("generate tax report");

        assert_eq!(report.disposals.len(), 1);
        assert_eq!(report.summary.disposals, 1);
        assert_eq!(report.disposals[0].tx_id, included_sell_id);
        assert_ne!(report.disposals[0].tx_id, excluded_sell_id);

        drop(harness);
    }
