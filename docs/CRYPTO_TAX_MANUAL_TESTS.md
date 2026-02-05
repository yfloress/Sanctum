# Crypto Tax — Manual QA Checklist

> Goal: verify the Crypto → Tax tab works end‑to‑end **offline-first** and without unintended network calls.

## 0) Prep
- Ensure the app is **offline** (disable network) and open a vault with some crypto transactions.
- Optional: have an IPC CSV ready (CSV format, not XLSX).

## 1) Navigation + Layout
- Open **Crypto → Tax** and confirm the tab scrolls (no content cut off).
- Resize the window to small width; verify cards and text reflow without clipping.
- Verify the IPC URL is inside a selectable field and can be highlighted.

## 2) Defaults & Persistence
- Default method should show **HIFO** selected when no settings exist.
- Change **Jurisdiction** (Chile/USA), change method, toggle swaps/fee‑crypto.
- Click **Save Tax Settings**.
- Restart the app and confirm all settings persist for the same period.

## 3) IPC Import (Chile)
- Click **Import IPC (CSV)** and select a valid CSV.
- Confirm success toast appears and IPC summary shows first/last month + count.
- Re‑import the same CSV; ensure no crash and summary remains correct.
- Try a malformed CSV: expect an error toast (no crash).

## 4) Report Generation
- Click **Generate Report**.
- Expect a summary line (disposals, proceeds, cost, gain).
- If Jurisdiction = USA, the summary should show **Short/Long** fields.
- If data is incomplete (missing prices), ensure warnings show a non‑empty count.

## 5) Export CSV
- Click **Export CSV**, pick a location, verify file is created.
- Open the CSV and confirm:
  - Header line exists
  - Disposal rows include tx id, date, symbol, proceeds, cost, gain
  - Warnings section appears when warnings exist

## 6) Offline‑First Check
- With network disabled, verify all actions still work.
- Confirm there are **no network requests** when importing CSV, generating report, or exporting.

## 7) Regression Spot‑Checks
- Open **Crypto → Portfolio** and **Wallets** to ensure no UI regression.
- Add a swap transaction and re‑generate report to verify swap handling.

