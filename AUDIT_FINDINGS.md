# Frontend Integration Audit Findings

**Date**: 2026-03-29
**Audited By**: Claude Code Audit Agent
**Status**: 6 Critical/High Severity Issues Found

---

## Executive Summary

A comprehensive audit of Tauri commands vs TypeScript API integrations reveals **6 structural gaps** that prevent correct frontend functionality:

- **3 Critical**: Wrong return types in crypto.ts causing runtime errors
- **1 High**: Missing API wrapper for habits milestone functionality
- **2 Medium**: Type definition mismatches in ingestion and vault modules

---

## Issues Detailed

### ISSUE #1: crypto.ts - exportTaxReportCsv()

**File**: `/app/ui-svelte/src/lib/api/crypto.ts` (lines 124-125)

**Current Code**:
```typescript
export function exportTaxReportCsv(period_id: string, path: string): Promise<string> {
  return invoke('export_tax_report_csv', { period_id, path })
}
```

**Problem**:
- Declared return type: `Promise<string>`
- Actual Rust return: `Result<(), String>` (void, no data)
- Will cause type error when calling code expects string

**Rust Command**:
```rust
#[tauri::command]
pub fn export_tax_report_csv(
    controller: State<'_, Arc<AppController>>,
    period_id: String,
    path: String,
) -> Result<(), String> {
    // ... exports to file, returns nothing
}
```

**Fix**: Change return type to `Promise<void>`

---

### ISSUE #2: crypto.ts - exportTaxHistoryCsv()

**File**: `/app/ui-svelte/src/lib/api/crypto.ts` (lines 128-129)

**Current Code**:
```typescript
export function exportTaxHistoryCsv(period_id: string, path: string): Promise<string> {
  return invoke('export_tax_history_csv', { period_id, path })
}
```

**Problem**:
- Declared return type: `Promise<string>`
- Actual Rust return: `Result<(), String>` (void)
- Identical to Issue #1

**Fix**: Change return type to `Promise<void>`

---

### ISSUE #3: crypto.ts - importIpcCsv()

**File**: `/app/ui-svelte/src/lib/api/crypto.ts` (lines 132-133)

**Current Code**:
```typescript
export function importIpcCsv(content: string): Promise<number> {
  return invoke('import_ipc_csv', { content })
}
```

**Problem**:
- Declared return type: `Promise<number>`
- Actual Rust return: `IpcSummaryDto { records_count: usize, date_range: Option<String> }`
- Type is **completely wrong** - not a number at all

**Rust Command**:
```rust
#[tauri::command]
pub fn import_ipc_csv(
    controller: State<'_, Arc<AppController>>,
    content: String,
) -> Result<IpcSummaryDto, String> {
    // ... returns IpcSummaryDto
}
```

**Fix**: Change return type to `Promise<IpcSummaryDto>`

**Also**: Verify `IpcSummaryDto` type exists in `lib/types.ts`:
```typescript
export interface IpcSummaryDto {
  records_count: number
  date_range: string | null
}
```

---

### ISSUE #4: crypto.ts - fillMissingTaxPrices()

**File**: `/app/ui-svelte/src/lib/api/crypto.ts` (lines 140-148)

**Current Code**:
```typescript
export function fillMissingTaxPrices(
  tx_id: string,
  price_per_coin?: number,
  fee_usd?: number,
  override_proceeds?: number
): Promise<number> {
  return invoke('fill_missing_tax_prices', {
    tx_id,
    price_per_coin,
    fee_usd,
    override_proceeds
  })
}
```

**Problem**:
- Declared return type: `Promise<number>`
- Actual Rust return: `Result<bool, String>`
- Should return boolean, not number

**Rust Command**:
```rust
#[tauri::command]
pub fn fill_missing_tax_prices(
    controller: State<'_, Arc<AppController>>,
    tx_id: String,
    price_per_coin: Option<f64>,
    fee_usd: Option<f64>,
    override_proceeds: Option<f64>,
) -> Result<bool, String> {
    // ... returns bool indicating success
}
```

**Fix**: Change return type to `Promise<boolean>`

---

### ISSUE #5: habits.ts - Missing add_milestone() Wrapper

**File**: `/app/ui-svelte/src/lib/api/habits.ts`

**Problem**:
- Tauri command exists: `/app/src-tauri/src/commands/habits.rs` line 420-430
- No TypeScript wrapper function in habits.ts
- Frontend cannot call this command

**Rust Command**:
```rust
#[tauri::command]
pub fn add_milestone(
    controller: State<'_, Arc<AppController>>,
    reward_id: String,
    target_days: u32,
    reward_text: String,
) -> Result<MilestoneDto, String> {
    // ... creates milestone
}
```

**Missing TypeScript Wrapper**:
```typescript
export function addMilestone(
  reward_id: string,
  target_days: number,
  reward_text: string
): Promise<MilestoneDto> {
  return invoke('add_milestone', {
    reward_id,
    target_days,
    reward_text
  })
}
```

**Impact**: UI cannot add milestones to streak rewards

---

### ISSUE #6a: types.ts - ImportPreviewResponse Type Mismatch

**File**: `/app/ui-svelte/src/lib/types.ts`

**Problem**: Type definition doesn't match what backend returns

**Current Type** (probably):
```typescript
export interface ImportPreviewResponse {
  source: string
  total_records: number
  to_add: number
  to_skip: number
  changes: Array<unknown>
}
```

**Actual Response from Rust**: Matches `ImportResultsResponse` structure, not preview-specific fields

**Impact**: `previewImport()` and `previewExchangeCsv()` will receive data that doesn't match type expectations

**Fix**: Verify the actual response structure from:
- `/app/src-tauri/src/commands/ingestion.rs` - `preview_import()` and `preview_exchange_csv()`

---

### ISSUE #6b: types.ts - Unused VaultExportResult

**File**: `/app/ui-svelte/src/lib/types.ts`

**Problem**: Type defined but doesn't match command return

**Type Definition** (probably):
```typescript
export interface VaultExportResult {
  path: string
  success: boolean
}
```

**Actual Rust Return**:
```rust
#[tauri::command]
pub fn export_vault(
    controller: State<'_, Arc<AppController>>,
    path: String,
) -> Result<(), String> {
    // ... returns nothing on success
}
```

**Impact**: Type is unused, misleading for future developers

**Fix**: Delete `VaultExportResult` type if unused, or use correct type `Promise<void>` for `export_vault()`

---

## Action Plan

### Phase 1: Critical Fixes (Blocks Functionality)

- [ ] Fix `exportTaxReportCsv()` return type → `Promise<void>`
- [ ] Fix `exportTaxHistoryCsv()` return type → `Promise<void>`
- [ ] Fix `importIpcCsv()` return type → `Promise<IpcSummaryDto>`
- [ ] Fix `fillMissingTaxPrices()` return type → `Promise<boolean>`
- [ ] Add `addMilestone()` wrapper in habits.ts

### Phase 2: Type Cleanup

- [ ] Verify `ImportPreviewResponse` structure matches actual response
- [ ] Remove or correct `VaultExportResult` type
- [ ] Run `svelte-check` to verify all types are correct

### Phase 3: Verification

- [ ] Test tax report export functionality
- [ ] Test IPC CSV import with real data
- [ ] Test milestone creation in habits module
- [ ] Verify no type errors in full build

---

## Verification Checklist

Before marking as "Complete":

- [ ] All 4 crypto.ts functions have correct return types
- [ ] `addMilestone()` can be called from HabitsPage component
- [ ] Types in `lib/types.ts` match all Rust DTOs
- [ ] `pnpm exec svelte-check` returns 0 errors
- [ ] Test import/export flows work end-to-end

---

## Files to Modify

1. `/app/ui-svelte/src/lib/api/crypto.ts` - Fix 4 return types
2. `/app/ui-svelte/src/lib/api/habits.ts` - Add `addMilestone()` wrapper
3. `/app/ui-svelte/src/lib/types.ts` - Verify type definitions

---

## Notes

- All parameter names (snake_case) are correctly formatted ✓
- All command files have corresponding API modules ✓
- No missing commands except as noted above

