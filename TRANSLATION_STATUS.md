# Translation Status — Svelte Frontend

> Tracks the dual work of: (1) wiring `i18n.t()` calls into Svelte components,
> and (2) updating locale files (`en.ftl` / `es.ftl`) to match the new frontend text.
>
> The locale files were written for the Slint frontend. ~80% of keys should carry over,
> but the Svelte frontend has different component structure, labels, and phrasing.

---

## How i18n Works

1. **Backend**: `get_translations` command reads `locales/{lang}.ftl` (Fluent format) and returns all key-value pairs.
2. **Store**: `i18n.svelte.ts` stores them in a reactive `$state` map, exposes `i18n.t(key, fallback)`, `i18n.tArgs(key, args)`, `i18n.tPlural(key, count)`.
3. **Language switch**: `SettingsPage` calls `set_preferred_language` → backend switches Fluent bundle → frontend calls `i18n.load()` to refresh strings.
4. **Status**: Sidebar and LoginPage already wired. Remaining pages still have hardcoded strings.

---

## ⚠️ MANDATORY WORKFLOW — READ → UPDATE → IMPLEMENT

**This order is non-negotiable. Never skip steps or change the order.**

### Step 1: READ the frontend code
- Open the Svelte page/component source
- Extract **every** hardcoded English string from the template
- Note the **exact text**, casing, and context of each string
- The frontend text is the source of truth — not the locale files

### Step 2: UPDATE the locale files
- Compare extracted strings against existing keys in `en.ftl` / `es.ftl`
- **If a key exists but text doesn't match** → update the key's value to match frontend
- **If no key exists** → add a new key with the frontend's text
- **If a key exists but is unused** → remove or mark for removal
- Update both `en.ftl` and `es.ftl` simultaneously

### Step 3: IMPLEMENT i18n.t() in the component
- Replace each hardcoded string with `i18n.t('key', 'Fallback')`
- The fallback MUST be the original English text (safety net)
- For parameterized strings use `i18n.tArgs('key', { var: value })`
- For plurals use `i18n.tPlural('key', count)`

> **WHY THIS ORDER?** The locale files were written for the old Slint frontend.
> The Svelte frontend has different text. If you implement first and force the
> frontend to use old locale text, you'll break the UI. Always read the actual
> frontend text first, update locales to match, then wire.

---

## Per-Page Status

| Page / Component | Read | Update | Implement | Notes |
|------------------|------|--------|-----------|-------|
| `Sidebar.svelte` | [x] | [x] | [x] | 7 labels wired (`5ad1d16`) |
| `LoginPage.svelte` | [x] | [x] | [x] | 8 strings wired (`5ad1d16`) |
| `SettingsPage.svelte` | [x] | [x] | [x] | ~45 strings wired (`f986045`). Includes import flow, toasts. |
| `DashboardPage.svelte` | [ ] | [ ] | [ ] | ~25 strings. Chart titles, stat labels, recent activity. |
| `FinancesPage.svelte` | [ ] | [ ] | [ ] | ~80 strings. Largest page. Tabs, modals, forms, categories. |
| `HabitsPage.svelte` | [ ] | [ ] | [ ] | ~60 strings. Heatmap, analytics, rewards, goals. |
| `CryptoPage.svelte` | [ ] | [ ] | [ ] | ~120 strings. Tax UI, modals, ticker config, portfolio. |
| `Toast.svelte` | [ ] | [ ] | [ ] | ~2 strings. Minimal. |

---

## Locale File Audit Checklist

### Keys likely needing updates (Slint → Svelte differences)

- [x] **Nav labels**: Changed from UPPERCASE to Title Case
- [x] **Login labels**: Rewritten to match Svelte text
- [ ] **Section headers**: Svelte uses different casing/wording
- [ ] **Button labels**: Svelte may use different text
- [ ] **Modal titles**: Svelte modals may have different titles
- [ ] **Empty states**: Svelte has different empty state messaging
- [ ] **Toast messages**: Error/success messages may differ
- [ ] **Chart labels**: ECharts labels are new (Slint didn't have ECharts)
- [ ] **Tab names**: Svelte tab structure differs from Slint

### Keys to add (new in Svelte, not in locale files)

- [ ] Cash flow chart labels (Dashboard)
- [ ] Overview/Activity/Settings tab labels (Finances)
- [ ] ECharts axis labels and tooltips
- [ ] Import flow: exchange detection, preview, confirm steps
- [ ] Any new modal/form text

### Keys to remove (Slint-only, not in Svelte)

- [ ] Slint-specific widget labels
- [ ] Slint panel/window titles
- [ ] Any keys referencing Slint components
