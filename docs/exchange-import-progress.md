# Exchange CSV Import -- Progress Tracker

> This file tracks implementation state for `feat/crypto-exchange-import`.
> If the AI session runs out of tokens, resume from here.

## Branch

`feat/crypto-exchange-import` (based on `dev` at `b6b33cc`)

## Architecture Overview

Exchange CSV parsers live in `src/features/ingestion/parsers/exchange/`.
Each parser converts an exchange-specific CSV into `Vec<ImportCryptoTransaction>`
(defined in `src/features/ingestion/types.rs`), which is the universal
intermediate representation. From there, the existing ingestion pipeline
handles validation, dedup, wallet/coin resolution, and DB insertion.

### Data flow

```
Exchange CSV file
  -> detect_exchange_format() identifies source
  -> ExchangeParser::parse(content, wallet_name) -> Vec<(usize, ImportCryptoTransaction)>
  -> IngestionService::process_crypto_transactions() (existing)
  -> DB
```

### Key type: ImportCryptoTransaction

Fields that exchange parsers must populate:
- `date`: ISO-8601 string (YYYY-MM-DD or YYYY-MM-DD HH:MM:SS)
- `wallet`: string (user provides or defaults to exchange name)
- `symbol`: uppercase ticker (BTC, ETH, XMR, etc.)
- `transaction_type`: one of `trade`, `income`, `expense`, `transfer`
- `amount`: f64, always positive
- `subtype`: specific action within type (buy, sell, swap, airdrop, deposit, withdrawal, staking, etc.)
- `price_per_coin`: Option<f64> -- USD price at tx time (if derivable)
- `fee`: Option<f64> -- fee in USD
- `fee_coin_symbol`: Option<String> -- if fee paid in crypto
- `fee_amount`: Option<f64> -- fee amount in crypto
- `swap_to_symbol` / `swap_to_amount`: for swaps
- `override_proceeds` / `override_cost_basis`: not usually set by parsers
- `notes`: Optional description/reference ID

### Sanctum type/subtype taxonomy (from tax/types.rs)

| type     | valid subtypes                                                        |
|----------|-----------------------------------------------------------------------|
| trade    | buy, sell, swap, other                                                |
| income   | interest, reward, airdrop, gift, staking, mining, fork, payment, rebate, other |
| expense  | payment, gift, fee, lost, stolen, donation, sell, other               |
| transfer | deposit, withdrawal                                                   |

## Exchanges to implement

### READY (have format specs)

- [x] Kraken Ledger CSV
- [x] Kraken Trades CSV
- [x] Binance All Statements CSV
- [x] Binance Spot Trade History CSV
- [x] Feather Wallet CSV (Monero)

### PENDING (need samples from user)

- [ ] CryptoMKT -- Chilean/LatAm exchange, no public CSV docs
- [ ] MEXC -- need full statement format (not just spot)
- [ ] Bybit -- need full statement format (not just spot)
- [ ] Cake Wallet -- multi-coin, format varies

## Implementation checklist

### Phase 1: Parser infrastructure
- [x] Create `src/features/ingestion/parsers/exchange/mod.rs` -- ExchangeSource enum, detect function, trait
- [x] Create `src/features/ingestion/parsers/exchange/common.rs` -- shared helpers (timestamp parsing, currency normalization)
- [x] Create `src/features/ingestion/parsers/exchange/kraken.rs` -- Kraken Ledger + Trades parser
- [x] Create `src/features/ingestion/parsers/exchange/binance.rs` -- Binance All Statements + Spot parser
- [x] Create `src/features/ingestion/parsers/exchange/feather.rs` -- Feather Wallet parser
- [x] Register module in `src/features/ingestion/parsers/mod.rs`

### Phase 2: Integration with ingestion service
- [x] Add `ImportFormat::ExchangeCsv(ExchangeSource)` variant to types.rs
- [x] Update `detect_format()` in parsers/mod.rs to detect exchange CSVs
- [x] Add import/preview methods in service.rs for exchange CSV
- [x] Add i18n keys for exchange import messages (locales/*.ftl)

### Phase 3: Tests
- [x] Unit tests for each parser with sample CSV data
- [x] Test currency normalization (Kraken XXBT->BTC, etc.)
- [x] Test ledger row pairing (Kraken trade/spend+receive refid linking)
- [x] Test Binance operation mapping (Convert pairing, etc.)
- [x] Test detect_format routes exchange CSVs correctly (parsers/mod.rs tests)
- [x] Fix pre-existing feather txid truncation test expectation

### Phase 4: UI integration
- [x] Add exchange import section to settings page (ui/pages/settings.slint)
- [x] Add IngestionAdapter properties/callbacks for exchange flow (ui/globals.slint)
- [x] Add translation properties for exchange import (ui/translations.slint)
- [x] Wire exchange import callbacks with file picker + auto-detection (src/ui/callbacks/ingestion.rs)
- [x] Route shared preview modal confirm/cancel via `is-exchange-import` flag (ui/app.slint)
- [x] Add controller methods: import_exchange_csv, preview_exchange_csv, detect_exchange_source
- [x] Show exchange-specific guidance hints (Kraken, Binance, Feather export instructions)

## Format specs

### Kraken Ledger CSV

Headers (v1): `txid,refid,time,type,subtype,aclass,asset,amount,fee,balance`
Headers (v2): `txid,refid,time,type,subtype,aclass,subclass,asset,wallet,amount,fee,balance`

Types and mapping to Sanctum:
- `trade` -> TWO rows per trade linked by `refid` (negative=outgoing, positive=incoming)
  -> Sanctum: `type=trade, subtype=buy|sell|swap`
- `deposit` -> `type=transfer, subtype=deposit`
- `withdrawal` -> `type=transfer, subtype=withdrawal`
- `staking` (positive) -> `type=income, subtype=staking`
- `earn` (positive, reward subtype) -> `type=income, subtype=staking`
- `spend` + `receive` pairs (same refid) -> `type=trade, subtype=buy|sell`
- `transfer` with staking subtypes (spottostaking, etc.) -> SKIP (internal)
- `adjustment`, `settled`, `margin trade`, `rollover` -> map by sign to trade/income/expense

Currency normalization:
XXBT/XBT->BTC, XETH->ETH, XLTC->LTC, XXMR->XMR, XXRP->XRP, XXLM->XLM,
XZEC->ZEC, XXDG/XDG->DOGE, XETC->ETC, XMLN->MLN, XREP->REP,
ZUSD->USD, ZEUR->EUR, ZCAD->CAD, ZGBP->GBP, ZJPY->JPY, ZAUD->AUD,
KFEE->FEE. Also strip `.S`, `.M` suffixes (staked variants).

### Kraken Trades CSV

Headers (v1): `txid,ordertxid,pair,time,type,ordertype,price,cost,fee,vol,margin,misc,ledgers`
Headers (v2): extended with aclass,subclass,posttxid,cprice,ccost,cfee,cvol,cmargin,net,trades

- Pair format: `BASE/QUOTE` (e.g. `BTC/USD`, `XXBTZUSD`)
- type: `buy` or `sell`
- `vol` = base amount, `cost` = quote amount, `fee` = fee in quote currency
- Mapping: `type=trade, subtype=buy|sell`

### Binance All Statements CSV

Headers: `User_ID,UTC_Time,Account,Operation,Coin,Change,Remark`

Operation mapping:
- `Buy`, `Transaction Buy` -> `type=trade, subtype=buy`
- `Sell`, `Transaction Sold` -> `type=trade, subtype=sell`
- `Binance Convert` -> paired rows (negative+positive) -> `type=trade, subtype=swap`
- `Small Assets Exchange BNB` -> `type=trade, subtype=swap` (tricky, may skip)
- `Deposit` -> `type=transfer, subtype=deposit`
- `Withdraw` -> `type=transfer, subtype=withdrawal`
- `Distribution`, `Airdrop Assets` -> `type=income, subtype=airdrop`
- `Staking Rewards` -> `type=income, subtype=staking`
- `Fee`, `Transaction Fee` -> `type=expense, subtype=fee`
- `Fiat Deposit` -> SKIP (fiat, not crypto)
- `Fiat Withdrawal` -> SKIP (fiat)
- `Transfer Between Main and Funding Wallet` -> SKIP (internal)
- `Binance Card Cashback` -> `type=income, subtype=rebate`
- `Binance Card Spending` -> `type=expense, subtype=payment`
- `Transaction Spend` -> paired with Transaction Revenue (trade)
- `Transaction Revenue` -> paired with Transaction Spend (trade)

Currency normalization: BCC->BCH, NANO->XNO
LUNA->LUNC if date < 2022-05-27

### Binance Spot Trade History CSV

Headers: `Date(UTC),Pair,Side,Price,Executed,Amount,Fee`

- Side: BUY or SELL
- Executed: amount in base currency with unit suffix (e.g. "0.5BTC")
- Amount: amount in quote currency with unit suffix
- Fee: fee with unit suffix
- Mapping: `type=trade, subtype=buy|sell`

### Feather Wallet CSV (Monero)

Headers: `blockheight,epoch,date,direction,amount,fee,txid,address,description,paymentid`

- direction: `in` -> `type=transfer, subtype=deposit`
- direction: `out` -> `type=transfer, subtype=withdrawal`
- symbol is always `XMR`
- fee is in XMR
- Notes populated from `description` and `txid`

## Files modified/created

### New files
- `docs/exchange-import-progress.md` (this file)
- `src/features/ingestion/parsers/exchange/mod.rs`
- `src/features/ingestion/parsers/exchange/common.rs`
- `src/features/ingestion/parsers/exchange/kraken.rs`
- `src/features/ingestion/parsers/exchange/binance.rs`
- `src/features/ingestion/parsers/exchange/feather.rs`

### Modified files
- `src/features/ingestion/parsers/mod.rs` -- add `pub mod exchange;`, re-exports, detect_format exchange detection, tests
- `src/features/ingestion/types.rs` -- add `ExchangeCsv(ExchangeSource)` variant to ImportFormat
- `src/features/ingestion/service.rs` -- add import/preview exchange CSV methods (explicit + auto-detect)
- `src/controller/ingestion.rs` -- add import_exchange_csv, preview_exchange_csv, detect_exchange_source
- `src/ui/callbacks/ingestion.rs` -- add exchange import callbacks (file picker, confirm, cancel, wallet name)
- `src/ui/callbacks/translations.rs` -- load exchange import translation keys
- `ui/globals.slint` -- add IngestionAdapter exchange properties (is-exchange-import, detected-format, wallet-name, etc.)
- `ui/translations.slint` -- add exchange import translation properties
- `ui/app.slint` -- route preview modal confirm/cancel via is-exchange-import flag
- `ui/pages/settings.slint` -- add Exchange Import section with hints
- `locales/en.ftl` -- add English exchange import keys
- `locales/es.ftl` -- add Spanish exchange import keys

### Dead code removed
- `BinanceOperation::TransactionFee` variant (never constructed; "Transaction Fee" maps to `Fee`)
- `BinanceOperation::TransactionFee` match arm in `single_row_to_transaction`
- `LedgerRow.txid`, `.refid`, `.subtype`, `.asset` fields (stored but never read from struct)
- Unused `txid` local variable in Kraken ledger parser
- Unused `non_empty` import in Kraken parser
- Unused `filename` field in `PendingExchangeImport`
- Simplified `subtype` internal-transfer check to inline `is_some_and`

### Test fix
- `feather::tests::notes_contain_description_and_truncated_txid`: expected `012defg` (7 chars)
  but truncation logic uses last 8 chars -> fixed to `c012defg`

## Notes

- Fiat currencies (USD, EUR, CLP, etc.) are SKIPPED by parsers -- Sanctum only
  tracks crypto transactions, not fiat movements.
- All parsers receive a `wallet_name: &str` parameter that the user provides
  (defaults to exchange name). This maps to the Sanctum wallet during ingestion.
- Kraken ledger parser must accumulate rows by `refid` before emitting trades,
  since a single trade generates 2+ rows. Incomplete pairs at EOF are emitted
  as single-sided transactions (deposit/withdrawal).
- Binance `Convert` operations also come as pairs (negative then positive Change
  with same timestamp). Parser accumulates and pairs them.
- All timestamps are normalized to UTC ISO-8601 format.
- Exchange-specific asset names are normalized to standard tickers before output.