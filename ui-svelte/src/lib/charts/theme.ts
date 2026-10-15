// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

import { app } from '../stores/app.svelte'

// ECharts options are configured in JS, so they cannot read the CSS theme
// variables. These helpers let each chart keep its exact dark-mode colors while
// substituting a cohesive light palette when light mode is active.

/** Light-mode chart "chrome" palette (axes, grid, tooltip, series accents). */
export const chartLight = {
  tooltipBg: '#ffffff',
  tooltipBorder: '#e7e4ea',
  tooltipText: '#211f26',
  /** Muted inline text inside tooltips (e.g. a date label). */
  muted: '#65636d',
  /** Axis baseline hairline. */
  axisLine: '#dbd8e0',
  /** Grid split lines. */
  splitLine: '#f2eff3',
  /** Bright axis labels. */
  label: '#65636d',
  /** Dimmed axis / legend labels. */
  labelDim: '#65636d',
  /** Gap border between pie / donut slices (matches the light card surface). */
  sliceBorder: '#ffffff',
  /** Donut centre emphasis label. */
  centerLabel: '#211f26',
  /** Series accent (purple). */
  accent: '#6e56cf',
  /** Positive / income / streak series (green). */
  positive: '#29a383',
  /** Negative / expense series (red). */
  negative: '#e5484d',
} as const

/**
 * Returns `darkVal` in dark mode and `lightVal` in light mode. Call this inside
 * a chart's `$derived` option so the chart re-themes reactively on toggle.
 */
export function pick<T>(darkVal: T, lightVal: T): T {
  return app.darkMode ? darkVal : lightVal
}
