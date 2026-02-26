# Exchange/Wallet Tax Audit and Stabilization Plan

Date: 2026-02-26
Owner: Ingestion + Tax pipeline
Status: Audit complete, implementation pending

## Progress update (2026-02-26)

- Done: source detection now scans a wider preamble window (up to 32 non-empty lines), reducing false negatives for report files with section titles/metadata before headers.
- Done: Kraken inverted-fiat parsing now keeps `fee_coin_symbol` and `fee_amount` strictly paired (both set or both empty).
- Done: batch import now skips `Binance Spot Trade History` when `Binance All Statements` is in the same batch to prevent overlap duplication.
- Done: batch import now skips `Kraken Trades` when `Kraken Ledger` is in the same batch to prevent overlap duplication.
- Done: ingestion service now uses additional price-agnostic dedup keys for overlap-prone exchange imports (Kraken/Binance/MEXC), reducing duplicate events when the same trade is re-imported with/without explicit price in separate sessions.
- Done: non-USD fiat quotes/fees are no longer injected as USD valuation inputs in Binance/Kraken/MEXC trade paths (kept as missing valuation instead of wrong USD).
- Done: this guard now also covers Binance All Statements paired conversions and MEXC Spot/Trade history fiat-quoted rows, with explicit non-USD quote annotations in notes.
- Done: missing-price tax warnings now differentiate non-USD quote normalization cases, and readiness UI surfaces a dedicated FX-normalization item.
- Done: automatic tax missing-price sync excludes non-USD quote warnings, avoiding accidental USD imputation for FX-priced trades.
- Added regression tests for all points above.

## 1) Scope

This document consolidates the current audit for exchange/wallet ingestion paths and defines a non-breaking plan to harden tax correctness.

In scope:
- Exchange CSV ingestion feeding crypto balances and tax engine
- Price/currency normalization used by tax calculations
- Dedup behavior when users import overlapping reports

Out of scope:
- New exchange integrations not yet in production
- UI redesign unrelated to ingestion/tax correctness

## 2) Baseline and constraint

Primary constraint:
- Do not break currently working balances and imports.

Operational rule:
- Changes must be additive, guarded, and validated per exchange before enabling globally.

## 3) Current audit findings (summary)

### Critical

1. Non-USD fiat values can enter tax math as if they were USD.
- `ImportCryptoTransaction.price_per_coin` is treated as USD by tax engine.
- Some parsers populate `price_per_coin` from non-USD fiat/stable pair prices without explicit normalization.
- Impact: inflated/deflated gains depending on quote currency.

2. Binance overlapping imports can duplicate trades.
- Importing `All Statements` + `Spot Trade History` together can produce duplicate economic events with different completeness.
- Impact: wrong balances and tax totals.

### High

3. Kraken fee parsing can attach fee coin without fee amount in one branch.
- This can trigger row skip/error paths depending on symbol resolution.
- Impact: dropped rows or inconsistent fees.

### Medium

4. MEXC statement parser intentionally skips spot/futures trading rows.
- If user only imports statement file, dataset can be incomplete.
- Impact: missing transactions unless complementary CSVs are also imported.

5. Source detection reads only first few non-empty lines.
- Files with long preambles/metadata can be misdetected.
- Impact: false "unsupported file" or skipped imports.

## 4) Exchange/Wallet risk map

- NotBank/CryptoMKT: partially mitigated with quote->USD anchoring; still requires strict invariant checks.
- Kraken: medium-high risk on fiat normalization + fee branch consistency.
- Binance: high risk on overlapping report dedup.
- MEXC: medium risk on "statement-only" imports.
- Wallet parsers (Feather/Monero GUI): lower risk for duplication; primary risk remains historical price gaps for fee disposals.

## 5) Stabilization plan (step-by-step)

## Phase 0 - Safety rails first

- Add invariants at ingestion boundary:
  - If `price_per_coin` is present, require explicit valuation currency metadata.
  - Reject/flag ambiguous rows instead of silently assuming USD.
- Add structured diagnostics (exchange, file type, row id, reason).
- Add regression fixtures for known problematic cases per exchange.

Exit criteria:
- No behavior changes yet, only visibility and guardrails.

## Phase 1 - Canonical monetary model

- Introduce a canonical internal representation:
  - `unit_price_native`
  - `unit_price_usd`
  - `native_quote_currency`
  - `valuation_source` (trade quote, derived, historical oracle, unknown)
- Keep backward compatibility by deriving legacy fields from canonical fields.

Exit criteria:
- Tax engine reads normalized USD value only when provenance is explicit.

## Phase 2 - Exchange-specific normalization adapters

- Kraken/Binance/MEXC/NotBank adapters must output canonical monetary model.
- Define per-exchange mapping specs for:
  - quote currency extraction
  - fiat/stable handling
  - fee coin + fee amount pairing
- Add adapter-level tests using realistic CSV fixtures.

Exit criteria:
- Each parser passes mapping tests and parity checks versus expected balances.

## Phase 3 - Overlap-safe dedup

- Add deterministic economic-event fingerprint:
  - timestamp (normalized)
  - base/quote assets
  - side
  - amount(s)
  - source exchange/account
- Add precedence rules for overlapping report types (prefer richer row).

Exit criteria:
- Importing combined report sets does not double count.

## Phase 4 - Price gap resolution flow

- Keep current "missing price" warnings, but classify causes:
  - truly missing market data
  - missing mapping/quote normalization
  - unresolved fee asset valuation
- Reuse cached resolved prices consistently.

Exit criteria:
- Warning count becomes explainable and reproducible.

## Phase 5 - Tax validation and hardening

- Add deterministic tax scenario tests (small synthetic portfolios with known output).
- Add jurisdiction sanity checks (Chile tax outputs and exports in CLP).
- Add property checks:
  - inventory cannot go negative unless explicitly allowed
  - fee disposal math does not exceed available lots

Exit criteria:
- Stable capital gains output across re-imports and method changes (HIFO/FIFO where applicable).

## Phase 6 - Rollout strategy

- Roll out per exchange behind internal toggle/order:
  1. NotBank
  2. Kraken
  3. Binance
  4. MEXC
- Keep old parser path as fallback during verification window.
- Promote only after parity checks pass on historical user fixtures.

Exit criteria:
- No regressions in balances, holdings, or tax reports.

## 6) Definition of done

- Same imported data -> same balances after repeated imports (idempotent).
- Overlapping files do not duplicate events.
- Tax totals are within expected tolerance versus manually verified fixtures.
- Warnings are actionable (clear reason + affected rows).
- No regressions for existing exchanges/wallets already working in production.

## 7) Recommended execution order for next implementation cycle

1. Phase 0 + Phase 1 foundations
2. Binance overlap dedup (highest user impact)
3. Kraken fee/fiat normalization
4. MEXC statement completeness UX + parser notes
5. Final tax validation pass with synthetic and real anonymized fixtures

## 8) Notes

- This document is intentionally implementation-focused and does not include sensitive user data.
- If a fixture includes personal/exported data, keep it outside git and use anonymized deterministic samples.
