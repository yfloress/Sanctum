import { app } from './stores/app.svelte'

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

