import { app } from './stores/app.svelte'

/** Placeholder shown in place of monetary values when balances are hidden. */
export const HIDDEN_PLACEHOLDER = '••••'

/**
 * Masks a pre-formatted monetary value when the global hide-balances toggle is on.
 * Use this on every personal money figure (balances, amounts, totals, P&L) so the
 * eye toggle hides them all at once. Leave public market data (e.g. coin prices) untouched.
 * @param value An already-formatted display string (or number)
 * @returns The value as-is, or the placeholder when balances are hidden
 */
export function mask(value: string | number): string {
  return app.hideBalances ? HIDDEN_PLACEHOLDER : String(value)
}

/**
 * Formats an amount the way the backend does, so a figure reads the same
 * wherever it comes from: `CLP 1.479.236` (no decimals, dot for thousands) and
 * `USD 1,234.56` for everything else.
 * @param value The amount in major units (pesos, dollars), not cents
 * @param currency Currency code; defaults to the preferred currency
 */
export function formatMoney(value: number, currency?: string): string {
  const code = (currency || app.settings?.preferred_currency || 'USD').toUpperCase()
  const abs = Math.abs(value)
  const body = code === 'CLP'
    ? Math.round(abs).toLocaleString('en-US').replace(/,/g, '.')
    : abs.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })
  return `${value < 0 ? '- ' : ''}${code} ${body}`
}

/**
 * Formats a numeric value into a currency string. Without `options` it matches
 * {@link formatMoney}; with them (e.g. compact chart axes) it uses Intl.
 * @param value The raw numeric amount
 * @param overrideCurrency Optional specific currency code (e.g. 'CLP')
 */
export function formatCurrency(
  value: number,
  overrideCurrency?: string,
  options?: Intl.NumberFormatOptions
): string {
  const currencyCode = overrideCurrency || app.settings?.preferred_currency || 'USD'
  if (!options) return formatMoney(value, currencyCode)
  const locale = navigator.language || 'en-US'

  try {
    return new Intl.NumberFormat(locale, {
      style: 'currency',
      currency: currencyCode,
      ...options
    }).format(value)
  } catch (e) {
    // Fallback if currency code is somehow invalid
    return new Intl.NumberFormat(locale, {
      style: 'currency',
      currency: 'USD',
      ...options
    }).format(value)
  }
}

