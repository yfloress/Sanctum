# Crypto Tax & Reporting (Chile + USA) — Implementation Requirements

Goal: add tax/reporting capabilities for crypto using selectable cost-basis methods and jurisdiction-specific rules, with per-period configuration and exportable reports.

## Scope
- Compute **realized gains** per disposal using FIFO/LIFO/HIFO/CPP.
- Support **Chile (SII)** and **USA (IRS)** rules.
- Allow **per-period** settings and report generation (e.g., yearly, custom range).
- Export summary + detailed CSV/JSON.

## Key Definitions
- **Lot**: an acquisition event (buy/transfer_in) with qty + cost.
- **Disposal**: a taxable outflow (sell, optional swap, optional fee-in-crypto).
- **Cost basis**: acquisition cost per lot, adjusted by method + jurisdiction rules.

## Required Decisions (UI defaults)
- Swaps taxable by default; toggleable per period.
- Fees handling: toggle per period (fee USD reduces proceeds / increases basis; fee in crypto treated as disposal).
- Period method locking: allow per-year method selection, optional “lock method per year”.

## Jurisdiction Rules
### Chile (SII)
- **Major value** = proceeds − **inflation‑adjusted** cost (IPC).
- Persona natural sin contabilidad: no fee deductions as cost.
- Con contabilidad completa: fees can be treated as deductible expense.

### USA (IRS)
- Crypto is **property**; sales/swaps are taxable dispositions.
- If no specific ID, FIFO is default. (We still allow FIFO/LIFO/HIFO/CPP but must record chosen method in report.)
- Include short/long‑term classification by holding period.

## Data Requirements
- Transactions must provide **price_per_coin** for buy/sell.
- **Swaps require price or FMV** to compute proceeds; otherwise mark incomplete and exclude from report.
- Fees in crypto need coin + amount to compute disposal if enabled.
- Date stored as ISO string is sufficient; tie-break with tx.id for deterministic ordering.

## Data Model Changes (minimal)
- Add optional transaction fields:
  - `tax_type`, `tax_subtype`
  - `override_proceeds`, `override_cost_basis`
- **Settings keys** (examples):
  - `crypto_tax_jurisdiction`
  - `crypto_tax_method_default`
  - `crypto_tax_method_YYYY`
  - `crypto_tax_include_swaps`
  - `crypto_tax_include_fee_crypto_disposal`
  - `crypto_tax_fee_rules` (pn_no_accounting | accounting)
  - `crypto_tax_period_lock_method`
- Optional: add `fmv_total_usd` or require `price_per_coin` for swap legs.
- Optional: add `tx_ref`/evidence field (can reuse notes if keeping minimal).

## Engine Requirements
- Build lots from inflows (buy/transfer_in).
- Apply disposals by method:
  - FIFO: oldest lots first
  - LIFO: newest lots first
  - HIFO: highest unit cost first
  - CPP: weighted average cost
- For swaps: treat “from” as disposal, “to” as acquisition at FMV.
- For transfers: move lots between wallets (no disposal unless toggle says taxable).
- For Chile: apply IPC adjustment to lot cost by month (month before buy vs month before sale).
- For USA: produce short/long term gains (>= 365 days).

## IPC Dataset (Chile)
- Add local IPC table (month → index). Must be offline and editable.
- **No in-app download**. User downloads manually and imports a CSV.
- Provide **offline import** (user selects local file) and **edit** capability.
  - CSV only (no XLSX parsing in app).
  - Suggested official source (manual download → convert to CSV → import):
    - INE series (IPC empalmado):  
      https://www.ine.gob.cl/docs/default-source/%C3%ADndice-de-precios-al-consumidor/cuadros-estadisticos/series-empalmadas-y-antecedentes-historicos/series-empalmadas-diciembre-2009-a-la-fecha/serie-hist%C3%B3rica-empalmada-ipc-diciembre-2009-a-la-fecha-xls.xlsx

## Reports
- Summary: total proceeds, total cost, total gain/loss.
- Detail: per disposal with lot breakdown, dates, method, jurisdiction, warnings.
- Export CSV/JSON with settings metadata (jurisdiction, method, toggles).
- Current implementation: **Generate Report** builds an in‑memory report and **Export CSV** writes a local file (no network).

## UI Changes
- New **Crypto → Tax/Reports** tab (or section in Crypto Settings).
- Period selector (year or custom range).
- Method selector with per‑period override.
- Toggle switches for swaps taxable, fee‑coin disposal, fee rules.
- Warnings list for incomplete data (missing price, missing IPC, etc.).

## Open Items
- Final decision: include staking/airdrops/mining as income events.
- Decide if swaps require price input in UI or allow “incomplete”.
- Confirm CLP vs USD report currency (default USD unless user chooses CLP).
