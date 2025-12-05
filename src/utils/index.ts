import type { AggregatedAsset, CryptoAsset } from "../types/index.ts";

// ==================== Date Utilities ====================

/** Obtiene la fecha local en formato YYYY-MM-DD sin problemas de timezone */
export function getLocalDateString(date: Date = new Date()): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

/** Formatea una fecha ISO a formato legible (e.g., "Jan 15, 2024") */
export function formatDate(isoDate: string): string {
  const [year, month, day] = isoDate.split("T")[0].split("-");
  const months = [
    "Jan",
    "Feb",
    "Mar",
    "Apr",
    "May",
    "Jun",
    "Jul",
    "Aug",
    "Sep",
    "Oct",
    "Nov",
    "Dec",
  ];
  return `${months[parseInt(month) - 1]} ${parseInt(day)}, ${year}`;
}

// ==================== Currency Formatters ====================

/** Configuración de monedas soportadas (solo USD + CLP) */
export const CURRENCIES = {
  USD: { symbol: "$", decimals: 2, locale: "en-US" },
  CLP: { symbol: "$", decimals: 0, locale: "es-CL" },
} as const;

export type CurrencyCode = keyof typeof CURRENCIES;

/** Formatea centavos a la moneda especificada */
export function formatAmount(
  cents: number,
  currency: CurrencyCode = "USD",
): string {
  const config = CURRENCIES[currency] || CURRENCIES.USD;
  const value = cents / 100;

  return value.toLocaleString(config.locale, {
    minimumFractionDigits: config.decimals,
    maximumFractionDigits: config.decimals,
  });
}

/** Formatea un valor con símbolo de moneda */
export function formatCurrency(
  cents: number,
  currency: CurrencyCode = "USD",
): string {
  const config = CURRENCIES[currency] || CURRENCIES.USD;
  return `${config.symbol}${formatAmount(cents, currency)}`;
}

/** Formatea un valor en USD con separadores de miles */
export function formatUSD(value: number): string {
  return value.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

/** Formatea cantidades de crypto con decimales apropiados */
export function formatCryptoAmount(amount: number, decimals = 8): string {
  if (amount >= 1) {
    return amount.toLocaleString(undefined, { maximumFractionDigits: 4 });
  }
  return amount.toLocaleString(undefined, {
    maximumFractionDigits: decimals,
  });
}

// ==================== Validation Utilities ====================

/** Valida que un coin ID tenga formato válido para CoinGecko */
export function isValidCoinId(coinId: string): boolean {
  if (!coinId || coinId.length > 64) return false;
  if (!/^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/.test(coinId)) return false;
  if (coinId.includes("--")) return false;
  return true;
}

// ==================== Crypto Utilities ====================

/** Enriquece assets agregados con precios actuales y calcula PnL */
export function enrichAssetsWithPrices(
  assets: AggregatedAsset[],
  prices: CryptoAsset[],
): AggregatedAsset[] {
  return assets.map((asset) => {
    const priceData = prices.find((a) => a.id === asset.coin_id);
    const currentPrice = priceData?.current_price ?? 0;
    const currentValue = asset.total_amount * currentPrice;
    const unrealizedPnl = currentValue - asset.total_cost_basis;
    const unrealizedPnlPercentage =
      asset.total_cost_basis > 0
        ? (unrealizedPnl / asset.total_cost_basis) * 100
        : 0;

    return {
      ...asset,
      current_price: currentPrice,
      current_value: currentValue,
      unrealized_pnl: unrealizedPnl,
      unrealized_pnl_percentage: unrealizedPnlPercentage,
    };
  });
}
