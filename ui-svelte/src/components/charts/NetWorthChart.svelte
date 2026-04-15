<!-- Sanctum — a privacy-first personal finance, crypto, and habits vault.
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
    if (range === '6M') return mon
    if (range === '1Y') return `${mon} '${String(y).slice(2)}`
    return String(y)
  }

  function formatMoney(cents: number): string {
    const dollars = cents / 100
    if (Math.abs(dollars) >= 1_000_000) return `$${(dollars / 1_000_000).toFixed(1)}M`
    if (Math.abs(dollars) >= 1_000) return `$${(dollars / 1_000).toFixed(0)}k`
    return `$${dollars.toFixed(0)}`
  }

  function labelInterval(): number {
    if (range === '1M') return 5   // ~every 6 days → 5 labels
    if (range === '6M') return 29  // ~monthly → 6 labels
    if (range === '1Y') return 59  // ~every 2 months → 6 labels
    return 364                      // ALL: ~yearly
  }

  let option = $derived({
    backgroundColor: 'transparent',
    grid: { left: 60, right: 16, top: 20, bottom: 32 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: '#1a1a1a',
      borderColor: '#2a2a2a',
      textStyle: { color: '#e0e0e0', fontSize: 12 },
      formatter: (params: { axisValue: string; value: number }[]) => {
        const pt = params[0]
        if (!pt) return ''
        const dollars = pt.value / 100
        return `<span style="color:#888;font-size:11px">${formatDate(pt.axisValue)}</span><br/><b>${formatCurrency(dollars)}</b>`
      },
    },
    xAxis: {
      type: 'category',
      data: data.dates,
      boundaryGap: false,
      axisLine: { lineStyle: { color: '#2a2a2a' } },
      axisTick: { show: false },
      axisLabel: {
        color: '#555',
        fontSize: 10,
        formatter: (val: string) => formatDate(val),
        interval: labelInterval(),
      },
    },
    yAxis: {
      type: 'value',
      axisLine: { show: false },
      axisTick: { show: false },
      splitLine: { lineStyle: { color: '#1a1a1a' } },
      axisLabel: {
        color: '#555',
        fontSize: 10,
        formatter: (val: number) => formatMoney(val),
      },
    },
    series: [{
      type: 'line',
      data: data.values,
      smooth: 0.4,
      showSymbol: false,
      lineStyle: { color: '#a855f7', width: 2 },
      areaStyle: {
        color: {
          type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(168, 85, 247, 0.25)' },
            { offset: 1, color: 'rgba(168, 85, 247, 0.02)' },
          ],
        },
      },
    }],
  })
</script>

<BaseChart {option} height="260px" />
