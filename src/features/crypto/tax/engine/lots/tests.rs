    use super::super::types::TaxPeriod;
    use super::*;
    use crate::features::crypto::TaxReportSummary;
    use chrono::NaiveDate;
    use std::collections::BTreeMap;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base_report() -> TaxReport {
        TaxReport {
            period_id: "2024".to_string(),
            period_start: "2024-01-01".to_string(),
            period_end: "2024-12-31".to_string(),
            jurisdiction: "usa".to_string(),
            method: "fifo".to_string(),
            summary: TaxReportSummary::default(),
            disposals: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn chile_report() -> TaxReport {
        TaxReport {
            period_id: "2024".to_string(),
            period_start: "2024-01-01".to_string(),
            period_end: "2024-12-31".to_string(),
            jurisdiction: "chile".to_string(),
            method: "fifo".to_string(),
            summary: TaxReportSummary::default(),
            disposals: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn test_period() -> TaxPeriod {
        TaxPeriod {
            id: "2024".to_string(),
            start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        }
    }

    fn tx(id: &str, kind: &str, amount: f64, price: f64, date: &str) -> CryptoTransaction {
        let (fiscal_type, subtype) = match kind {
            "buy" => ("trade", Some("buy")),
            "sell" => ("trade", Some("sell")),
            "swap" => ("trade", Some("swap")),
            "transfer_in" => ("transfer", Some("deposit")),
            "transfer_out" => ("transfer", Some("withdrawal")),
            _ => ("trade", None),
        };
        let mut tx = CryptoTransaction::new(
            id.to_string(),
            "wallet".to_string(),
            "btc".to_string(),
            "BTC".to_string(),
            fiscal_type.to_string(),
            amount,
            Some(price),
            None,
            date.to_string(),
            None,
        );
        tx.subtype = subtype.map(str::to_string);
        tx
    }

    fn tx_with_fee(
        id: &str,
        kind: &str,
        amount: f64,
        price: f64,
        fee: f64,
        date: &str,
    ) -> CryptoTransaction {
        let (fiscal_type, subtype) = match kind {
            "buy" => ("trade", Some("buy")),
            "sell" => ("trade", Some("sell")),
            "swap" => ("trade", Some("swap")),
            "transfer_in" => ("transfer", Some("deposit")),
            "transfer_out" => ("transfer", Some("withdrawal")),
            _ => ("trade", None),
        };
        let mut tx = CryptoTransaction::new(
            id.to_string(),
            "wallet".to_string(),
            "btc".to_string(),
            "BTC".to_string(),
            fiscal_type.to_string(),
            amount,
            Some(price),
            Some(fee),
            date.to_string(),
            None,
        );
        tx.subtype = subtype.map(str::to_string);
        tx
    }

    // -----------------------------------------------------------------------
    // HIFO ordering
    // -----------------------------------------------------------------------

    #[test]
    fn hifo_uses_highest_cost_lot() {
        let mut report = base_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();
        let period = test_period();
        let ipc_map = BTreeMap::new();

        let buy1 = tx("b1", "buy", 1.0, 100.0, "2024-01-05");
        let buy2 = tx("b2", "buy", 1.0, 200.0, "2024-02-05");

        add_lot(
            &mut report,
            &mut lots,
            &buy1,
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            TaxJurisdiction::Usa,
            None,
        );
        add_lot(
            &mut report,
            &mut lots,
            &buy2,
            NaiveDate::from_ymd_opt(2024, 2, 5).unwrap(),
            TaxJurisdiction::Usa,
            None,
        );

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Hifo,
            jurisdiction: TaxJurisdiction::Usa,
            ipc_map: &ipc_map,
        };

        let req = DisposalRequest {
            coin_id: "btc",
            amount: 1.0,
            sale_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            tx_id: "s1",
            proceeds: 300.0,
            taxable: true,
        };

        let (allocs, cost, _, _) = consume_lots(&mut report, &mut lots, &cfg, &req);

        assert_eq!(allocs.len(), 1);
        assert_eq!(allocs[0].allocation.lot_id, "b2");
        assert!(approx_eq(cost, 200.0));
    }

    #[test]
    fn chile_hifo_uses_highest_ipc_adjusted_cost_lot() {
        let mut report = chile_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();
        let period = test_period();
        let mut ipc_map = BTreeMap::new();

        // Sale in Aug => sale_prev = 2024-07
        // b1 nominal 100 but adjusted to 111.11 (100 * 100/90)
        // b2 nominal 105 and adjusted remains 105 (105 * 100/100)
        ipc_map.insert("2023-12".to_string(), 90.0);
        ipc_map.insert("2024-06".to_string(), 100.0);
        ipc_map.insert("2024-07".to_string(), 100.0);

        let buy1 = tx("b1", "buy", 1.0, 100.0, "2024-01-05");
        let buy2 = tx("b2", "buy", 1.0, 105.0, "2024-07-05");

        add_lot(
            &mut report,
            &mut lots,
            &buy1,
            NaiveDate::from_ymd_opt(2024, 1, 5).expect("valid date"),
            TaxJurisdiction::Chile,
            None,
        );
        add_lot(
            &mut report,
            &mut lots,
            &buy2,
            NaiveDate::from_ymd_opt(2024, 7, 5).expect("valid date"),
            TaxJurisdiction::Chile,
            None,
        );

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Hifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc_map,
        };

        let req = DisposalRequest {
            coin_id: "btc",
            amount: 1.0,
            sale_date: NaiveDate::from_ymd_opt(2024, 8, 1).expect("valid date"),
            tx_id: "s1",
            proceeds: 200.0,
            taxable: true,
        };

        let (allocs, cost, _, _) = consume_lots(&mut report, &mut lots, &cfg, &req);
        assert_eq!(allocs.len(), 1);
        assert_eq!(allocs[0].allocation.lot_id, "b1");
        assert!(cost > 110.0 && cost < 112.0);
    }

    // -----------------------------------------------------------------------
    // CPP (weighted average)
    // -----------------------------------------------------------------------

    #[test]
    fn cpp_uses_weighted_average_cost() {
        let mut report = base_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();
        let period = test_period();
        let ipc_map = BTreeMap::new();

        let buy1 = tx("b1", "buy", 1.0, 100.0, "2024-01-05");
        let buy2 = tx("b2", "buy", 3.0, 300.0, "2024-02-05");

        add_lot(
            &mut report,
            &mut lots,
            &buy1,
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            TaxJurisdiction::Usa,
            None,
        );
        add_lot(
            &mut report,
            &mut lots,
            &buy2,
            NaiveDate::from_ymd_opt(2024, 2, 5).unwrap(),
            TaxJurisdiction::Usa,
            None,
        );

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Cpp,
            jurisdiction: TaxJurisdiction::Usa,
            ipc_map: &ipc_map,
        };

        let req = DisposalRequest {
            coin_id: "btc",
            amount: 2.0,
            sale_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            tx_id: "s1",
            proceeds: 600.0,
            taxable: true,
        };

        let (_, cost, _, _) = consume_lots(&mut report, &mut lots, &cfg, &req);

        // 1×100 + 3×300 = 1000 total for 4 units → avg 250/unit → 2 × 250 = 500
        assert!(approx_eq(cost, 500.0));
    }

    // -----------------------------------------------------------------------
    // IPC cost adjustment (first adjustment)
    // -----------------------------------------------------------------------

    #[test]
    fn ipc_cost_adjustment_applies_for_chile() {
        let mut report = chile_report();
        let mut ipc = BTreeMap::new();
        ipc.insert("2023-12".to_string(), 100.0);
        ipc.insert("2024-01".to_string(), 110.0);
        let period = test_period();

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc,
        };

        let (adjusted, cost) =
            apply_ipc_cost_adjustment(&mut report, &cfg, "2023-12", "2024-01", 100.0, "tx1", true);

        assert!(adjusted.map(|v| approx_eq(v, 110.0)).unwrap_or(false));
        assert!(approx_eq(cost, 110.0));
    }

    #[test]
    fn ipc_missing_emits_warning() {
        let mut report = chile_report();
        let ipc = BTreeMap::new();
        let period = test_period();

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc,
        };

        let (adjusted, cost) =
            apply_ipc_cost_adjustment(&mut report, &cfg, "2023-12", "2024-01", 100.0, "tx1", true);

        assert!(adjusted.is_none());
        assert!(approx_eq(cost, 100.0));
    }

    // -----------------------------------------------------------------------
    // IPC gain adjustment (second adjustment)
    // -----------------------------------------------------------------------

    #[test]
    fn gain_ipc_adjustment_applies_for_chile() {
        let mut report = chile_report();
        let mut ipc = BTreeMap::new();
        // Sale in August → sale_prev = "2024-07"
        // End of year prev = "2024-11" (November)
        ipc.insert("2024-07".to_string(), 120.0);
        ipc.insert("2024-11".to_string(), 122.0);
        let period = test_period();

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc,
        };

        let sale_date = NaiveDate::from_ymd_opt(2024, 8, 15).unwrap();
        let adjusted = apply_gain_ipc_adjustment(&mut report, &cfg, sale_date, 354.5, "tx1");

        // 354.5 × (122 / 120) = 354.5 × 1.01667 ≈ 360.41
        let expected = 354.5 * (122.0 / 120.0);
        assert!(approx_eq(adjusted, expected));
    }

    #[test]
    fn gain_ipc_adjustment_noop_for_december_sale() {
        let mut report = chile_report();
        let mut ipc = BTreeMap::new();
        // Sale in December → sale_prev = "2024-11" == year_end_prev
        ipc.insert("2024-11".to_string(), 122.0);
        let period = test_period();

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc,
        };

        let sale_date = NaiveDate::from_ymd_opt(2024, 12, 10).unwrap();
        let adjusted = apply_gain_ipc_adjustment(&mut report, &cfg, sale_date, 500.0, "tx1");

        assert!(approx_eq(adjusted, 500.0));
    }

    #[test]
    fn gain_ipc_adjustment_noop_for_usa() {
        let mut report = base_report();
        let ipc = BTreeMap::new();
        let period = test_period();

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Usa,
            ipc_map: &ipc,
        };

        let sale_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let adjusted = apply_gain_ipc_adjustment(&mut report, &cfg, sale_date, 200.0, "tx1");

        assert!(approx_eq(adjusted, 200.0));
    }

    // -----------------------------------------------------------------------
    // Chile: fees excluded from cost basis
    // -----------------------------------------------------------------------

    #[test]
    fn chile_fee_excluded_from_cost_basis() {
        let mut report = chile_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();

        let buy = tx_with_fee("b1", "buy", 1.0, 100.0, 5.0, "2024-01-10");

        add_lot(
            &mut report,
            &mut lots,
            &buy,
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            TaxJurisdiction::Chile,
            None,
        );

        let lot = &lots["btc"][0];
        // Chile: cost = amount × price (no fee) = 1.0 × 100.0 = 100.0
        assert!(approx_eq(lot.unit_cost, 100.0));
    }

    #[test]
    fn usa_fee_included_in_cost_basis() {
        let mut report = base_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();

        let buy = tx_with_fee("b1", "buy", 1.0, 100.0, 5.0, "2024-01-10");

        add_lot(
            &mut report,
            &mut lots,
            &buy,
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            TaxJurisdiction::Usa,
            None,
        );

        let lot = &lots["btc"][0];
        // USA: cost = (amount × price) + fee = 100.0 + 5.0 = 105.0
        assert!(approx_eq(lot.unit_cost, 105.0));
    }

    #[test]
    fn chile_sale_fee_does_not_reduce_proceeds() {
        let mut report = chile_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();
        let period = test_period();
        let ipc_map = BTreeMap::new();

        let buy = tx("b1", "buy", 1.0, 100.0, "2024-01-10");
        add_lot(
            &mut report,
            &mut lots,
            &buy,
            NaiveDate::from_ymd_opt(2024, 1, 10).expect("valid date"),
            TaxJurisdiction::Chile,
            None,
        );

        let sell = tx_with_fee("s1", "sell", 1.0, 200.0, 10.0, "2024-02-10");
        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc_map,
        };
        apply_disposal(
            &mut report,
            &mut lots,
            &cfg,
            &sell,
            NaiveDate::from_ymd_opt(2024, 2, 10).expect("valid date"),
            true,
        );

        assert_eq!(report.disposals.len(), 1);
        assert!(approx_eq(report.disposals[0].proceeds, 200.0));
    }

    #[test]
    fn usa_sale_fee_reduces_proceeds() {
        let mut report = base_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();
        let period = test_period();
        let ipc_map = BTreeMap::new();

        let buy = tx("b1", "buy", 1.0, 100.0, "2024-01-10");
        add_lot(
            &mut report,
            &mut lots,
            &buy,
            NaiveDate::from_ymd_opt(2024, 1, 10).expect("valid date"),
            TaxJurisdiction::Usa,
            None,
        );

        let sell = tx_with_fee("s1", "sell", 1.0, 200.0, 10.0, "2024-02-10");
        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Usa,
            ipc_map: &ipc_map,
        };
        apply_disposal(
            &mut report,
            &mut lots,
            &cfg,
            &sell,
            NaiveDate::from_ymd_opt(2024, 2, 10).expect("valid date"),
            true,
        );

        assert_eq!(report.disposals.len(), 1);
        assert!(approx_eq(report.disposals[0].proceeds, 190.0));
    }

    // -----------------------------------------------------------------------
    // Chile: airdrops / staking have zero cost
    // -----------------------------------------------------------------------

    #[test]
    fn chile_airdrop_has_zero_cost() {
        let mut report = chile_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();

        // Airdrop with a known market price — Chile should still use $0.
        let airdrop = tx("a1", "buy", 10.0, 50.0, "2024-03-01");

        add_lot(
            &mut report,
            &mut lots,
            &airdrop,
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            TaxJurisdiction::Chile,
            Some("airdrop"),
        );

        let lot = &lots["btc"][0];
        assert!(approx_eq(lot.unit_cost, 0.0));
        // No "missing_price" warning should be emitted for zero-cost items.
        assert!(!report.warnings.iter().any(|w| w.code == "missing_price"));
    }

    #[test]
    fn chile_staking_has_zero_cost() {
        let mut report = chile_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();

        let staking = tx("s1", "buy", 5.0, 200.0, "2024-04-01");

        add_lot(
            &mut report,
            &mut lots,
            &staking,
            NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            TaxJurisdiction::Chile,
            Some("staking"),
        );

        let lot = &lots["btc"][0];
        assert!(approx_eq(lot.unit_cost, 0.0));
    }

    #[test]
    fn usa_airdrop_uses_fmv() {
        let mut report = base_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();

        let airdrop = tx("a1", "buy", 10.0, 50.0, "2024-03-01");

        add_lot(
            &mut report,
            &mut lots,
            &airdrop,
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            TaxJurisdiction::Usa,
            Some("airdrop"),
        );

        let lot = &lots["btc"][0];
        // USA: FMV is cost basis = 10.0 × 50.0 = 500.0 → 50.0/unit
        assert!(approx_eq(lot.unit_cost, 50.0));
    }

    // -----------------------------------------------------------------------
    // Full Chile example from LedgiFi guide
    // -----------------------------------------------------------------------

    #[test]
    fn chile_full_example_fifo_with_double_ipc() {
        // Reproduces the LedgiFi example:
        // Buy 1 BTC on 2024-01-05 at $1,000
        // Buy 1 BTC on 2024-02-21 at $1,200
        // Sell 1.5 BTC in August 2024 at $2,000
        // IPC adjustments bring the cost up and the gain is further adjusted.

        let mut report = chile_report();
        let mut lots: HashMap<String, Vec<Lot>> = HashMap::new();
        let period = test_period();

        let mut ipc = BTreeMap::new();
        // Made-up IPC values that match the LedgiFi percentages:
        // Jan cost adj 3.1% → IPC dec2023=100, IPC jul2024=103.1
        // Feb cost adj 2.4% → IPC jan2024=100, IPC jul2024=102.4
        // Gain adj 1.6% → IPC jul2024 → IPC nov2024
        ipc.insert("2023-12".to_string(), 100.0);
        ipc.insert("2024-01".to_string(), 100.0);
        ipc.insert("2024-07".to_string(), 103.1); // sale_prev for August sale
        ipc.insert("2024-11".to_string(), 104.75); // ~1.6% above 103.1

        // For the Feb buy, we need IPC jan → jul: 100 → 102.4
        // But we already have 2024-01 = 100.0 and 2024-07 = 103.1.
        // The LedgiFi example uses different % per lot. Let's use exact ratios.
        // To match exactly: buy1 cost adj = 1000 × (103.1/100) = 1031
        //                   buy2 cost adj = 1200 × (103.1/100) = 1237.2
        // But LedgiFi says buy2 adj = 1229 (2.4%), meaning IPC jan=100, IPC jul=102.4
        // This shows each lot uses its OWN buy_prev month.
        // buy1: prev_month = 2023-12 (dec), sale_prev = 2024-07 → 100 → 103.1 ✓
        // buy2: prev_month = 2024-01 (jan), sale_prev = 2024-07 → 100 → 103.1
        //   but LedgiFi says 2.4% for buy2, not 3.1%. This is because each month
        //   has different IPC values. Let's adjust to match the example exactly.
        // We'll set IPC values to produce the exact LedgiFi numbers.
        let mut ipc_exact = BTreeMap::new();
        ipc_exact.insert("2023-12".to_string(), 100.0); // buy1 prev
        ipc_exact.insert("2024-01".to_string(), 100.0); // buy2 prev
        ipc_exact.insert("2024-07".to_string(), 103.1); // sale prev (Aug sale)
        ipc_exact.insert("2024-11".to_string(), 104.7496); // gain adj: 1.6% above 103.1

        // Recalculate: buy2 adj would be 1200 × (103.1/100) = 1237.2 not 1229.
        // The difference is that LedgiFi uses DIFFERENT IPC per buy month.
        // Let's just use 102.4 for buy2's ratio → IPC jan=100, IPC jul would need
        // to be 102.4 for buy2. But it's the SAME sale month for both.
        // Actually the IPC is a single series. The difference in % comes from
        // different buy months having different IPC values.
        // buy1 dec2023 → jul2024: 3.1% means IPC_dec = X, IPC_jul = X × 1.031
        // buy2 jan2024 → jul2024: 2.4% means IPC_jan = Y, IPC_jul = Y × 1.024
        // So IPC_dec/IPC_jan ratio = (IPC_jul/1.031) / (IPC_jul/1.024)
        // = 1.024/1.031 ≈ 0.9932
        // Let's set: IPC_dec=99.32, IPC_jan=100.0, IPC_jul=102.4
        let mut ipc_ledgifi = BTreeMap::new();
        ipc_ledgifi.insert("2023-12".to_string(), 99.3219); // so 102.4/99.3219 ≈ 1.031
        ipc_ledgifi.insert("2024-01".to_string(), 100.0);
        ipc_ledgifi.insert("2024-07".to_string(), 102.4);
        ipc_ledgifi.insert("2024-11".to_string(), 104.0384); // 102.4 × 1.016

        let cfg = TaxConfig {
            period: &period,
            method: TaxMethod::Fifo,
            jurisdiction: TaxJurisdiction::Chile,
            ipc_map: &ipc_ledgifi,
        };

        let buy1 = tx("b1", "buy", 1.0, 1000.0, "2024-01-05");
        let buy2 = tx("b2", "buy", 1.0, 1200.0, "2024-02-21");

        add_lot(
            &mut report,
            &mut lots,
            &buy1,
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            TaxJurisdiction::Chile,
            None,
        );
        add_lot(
            &mut report,
            &mut lots,
            &buy2,
            NaiveDate::from_ymd_opt(2024, 2, 21).unwrap(),
            TaxJurisdiction::Chile,
            None,
        );

        // Sell 1.5 BTC in August
        let mut sell = tx("s1", "sell", 1.5, 1333.3333, "2024-08-15");
        // Total proceeds = 1.5 × 1333.3333 ≈ 2000
        sell.override_proceeds = Some(2000.0);

        let sale_date = NaiveDate::from_ymd_opt(2024, 8, 15).unwrap();
        apply_disposal(&mut report, &mut lots, &cfg, &sell, sale_date, true);

        assert_eq!(report.disposals.len(), 1);
        let d = &report.disposals[0];

        // FIFO: consume 1.0 BTC from buy1 + 0.5 BTC from buy2
        // buy1 adj cost: 1000 × (102.4 / 99.3219) ≈ 1031.0
        // buy2 adj cost for 0.5: 0.5 × 1200 × (102.4 / 100.0) = 0.5 × 1228.8 ≈ 614.4
        // total cost ≈ 1031 + 614.4 = 1645.4
        // raw gain = 2000 - 1645.4 = 354.6
        // gain adj = 354.6 × (104.0384 / 102.4) ≈ 354.6 × 1.016 ≈ 360.3
        assert!(d.proceeds == 2000.0);
        assert!(d.cost_basis > 1640.0 && d.cost_basis < 1650.0);
        // The gain should be the IPC-adjusted value, roughly ~360
        assert!(d.gain > 355.0 && d.gain < 365.0);

        // term should be None for Chile
        assert!(d.term.is_none());
    }
