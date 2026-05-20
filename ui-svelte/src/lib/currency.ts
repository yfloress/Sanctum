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
 * Formats a numeric value into a localized currency string.
 * Uses the user's preferred currency from settings by default.
 * @param value The raw numeric amount
 * @param overrideCurrency Optional specific currency code (e.g. 'CLP')
 * @returns Localized and formatted currency string
 */
export function formatCurrency(
  value: number,
  overrideCurrency?: string,
  options?: Intl.NumberFormatOptions
): string {
  const currencyCode = overrideCurrency || app.settings?.preferred_currency || 'USD'
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

