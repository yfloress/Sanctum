<!-- Sanctum — a privacy-first personal finance and crypto vault.
     Copyright (C) 2026  Kyronix

     This program is free software: you can redistribute it and/or modify
     it under the terms of the GNU Affero General Public License as
     published by the Free Software Foundation, either version 3 of the
     License, or (at your option) any later version.

     This program is distributed in the hope that it will be useful,
     but WITHOUT ANY WARRANTY; without even the implied warranty of
     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
     GNU Affero General Public License for more details.

     You should have received a copy of the GNU Affero General Public License
     along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>. -->

<script lang="ts">
  import BaseChart from './BaseChart.svelte'
  import { chartLight as L, pick } from '../../lib/charts/theme'
  import type { NetWorthChartData } from '../../lib/types'
  import { formatCurrency } from '../../lib/currency'

  interface Props {
    data: NetWorthChartData
    range?: string
  }

  let { data, range = '1M' }: Props = $props()

  const MONTHS = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec']

  function formatDate(iso: string): string {
    if (!iso || !iso.includes('-')) return iso
    const parts = iso.split('-')
    const y = Number(parts[0])
    const m = Number(parts[1])
    const d = Number(parts[2])
    const mon = MONTHS[(m - 1) % 12] ?? ''
    if (range === '1M') return `${mon} ${d}`
    if (range === '3M') return mon
    if (range === '6M') return mon
    if (range === '1Y') return `${mon} '${String(y).slice(2)}`
    return String(y)
  }

  function formatYAxis(value: number): string {
    if (value === 0) return formatCurrency(0)
    return formatCurrency(value, undefined, {
      notation: 'compact',
      compactDisplay: 'short',
      minimumFractionDigits: 0,
      maximumFractionDigits: 1,
    })
  }

  function labelInterval(): number {
    if (range === '1M') return 5   // ~every 6 days → 5 labels
    if (range === '3M') return 14  // ~every 2 weeks → 6 labels
    if (range === '6M') return 29  // ~monthly → 6 labels
    if (range === '1Y') return 59  // ~every 2 months → 6 labels
    return 364                      // ALL: ~yearly
  }

  let option = $derived({
    backgroundColor: 'transparent',
    grid: { left: 60, right: 16, top: 20, bottom: 32 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: pick('#1a1a1a', L.tooltipBg),
      borderColor: pick('#2a2a2a', L.tooltipBorder),
      textStyle: { color: pick('#e0e0e0', L.tooltipText), fontSize: 12 },
      formatter: (params: { axisValue: string; value: number }[]) => {
        const pt = params[0]
        if (!pt) return ''
        return `<span style="color:${pick('#888', L.muted)};font-size:11px">${formatDate(pt.axisValue)}</span><br/><b>${formatCurrency(pt.value)}</b>`
      },
    },
    xAxis: {
      type: 'category',
      data: data.dates,
      boundaryGap: false,
      axisLine: { lineStyle: { color: pick('#2a2a2a', L.axisLine) } },
      axisTick: { show: false },
      axisLabel: {
        color: pick('#555', L.labelDim),
        fontSize: 10,
        formatter: (val: string) => formatDate(val),
        interval: labelInterval(),
      },
    },
    yAxis: {
      type: 'value',
      axisLine: { show: false },
      axisTick: { show: false },
      splitLine: { lineStyle: { color: pick('#1a1a1a', L.splitLine) } },
      axisLabel: {
        color: pick('#555', L.labelDim),
        fontSize: 10,
        formatter: (val: number) => formatYAxis(val),
      },
    },
    series: [{
      type: 'line',
      data: data.values,
      smooth: 0.4,
      showSymbol: false,
      lineStyle: { color: pick('#a855f7', L.accent), width: 2 },
      areaStyle: {
        color: {
          type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
          colorStops: [
            { offset: 0, color: pick('rgba(168, 85, 247, 0.25)', 'rgba(139, 92, 246, 0.2)') },
            { offset: 1, color: pick('rgba(168, 85, 247, 0.02)', 'rgba(139, 92, 246, 0.02)') },
          ],
        },
      },
    }],
  })
</script>

<BaseChart {option} height="260px" />
