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
  import { chartLight as L, pick } from '../../lib/charts/theme'
  import type { PortfolioTrendData } from '../../lib/types'
  import { formatCurrency } from '../../lib/currency'

  interface Props {
    data: PortfolioTrendData
  }

  let { data }: Props = $props()

  function formatYAxis(value: number): string {
    if (value === 0) return formatCurrency(0)
    return formatCurrency(value, undefined, {
      notation: 'compact',
      compactDisplay: 'short',
      minimumFractionDigits: 0,
      maximumFractionDigits: 1,
    })
  }

  // Soften step corners by inserting a "shoulder" point just before each anchor:
  // same Y as previous anchor, X at 96% of the gap. With smooth enabled, plateaus
  // stay flat (consecutive same-Y points) and only the rising/falling segment
  // between shoulder and anchor gets bezier-rounded.
  const seriesData = $derived.by(() => {
    if (!data.dates.length) return [] as { value: [number, number]; symbol?: string }[]
    const showAnchorDots = data.values.length <= 3
    const out: { value: [number, number]; symbol?: string }[] = []
    for (let i = 0; i < data.dates.length; i++) {
      const t = new Date(data.dates[i] + 'T00:00:00').getTime()
      const v = data.values[i]
      if (i > 0) {
        const prevT = new Date(data.dates[i - 1] + 'T00:00:00').getTime()
        const prevV = data.values[i - 1]
        const gap = t - prevT
        if (gap > 0) {
          out.push({ value: [t - gap * 0.04, prevV], symbol: 'none' })
        }
      }
      out.push({ value: [t, v], symbol: showAnchorDots ? 'circle' : 'none' })
    }
    return out
  })

  let option = $derived({
    backgroundColor: 'transparent',
    grid: { left: 60, right: 20, top: 20, bottom: 30 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: pick('#1a1a1a', L.tooltipBg),
      borderColor: pick('#333', L.tooltipBorder),
      textStyle: { color: pick('#e0e0e0', L.tooltipText), fontSize: 12 },
      formatter: (params: { value: [number, number] }[]) => {
        const pt = params[0]
        if (!pt) return ''
        const v = Array.isArray(pt.value) ? pt.value[1] : pt.value
        return `<b>${formatCurrency(v)}</b>`
      },
    },
    xAxis: {
      type: 'time',
      axisLine: { lineStyle: { color: pick('#333', L.axisLine) } },
      axisLabel: { color: pick('#666', L.labelDim), fontSize: 10 },
    },
    yAxis: {
      type: 'value',
      axisLine: { show: false },
      splitLine: { lineStyle: { color: pick('#1a1a1a', L.splitLine) } },
      axisLabel: { color: pick('#666', L.labelDim), fontSize: 10, formatter: (val: number) => formatYAxis(val) },
    },
    series: [{
      type: 'line',
      data: seriesData,
      smooth: 0.4,
      showSymbol: true,
      symbolSize: 6,
      itemStyle: { color: pick('#4ade80', L.positive) },
      lineStyle: { color: pick('#4ade80', L.positive), width: 2.5, cap: 'round', join: 'round' },
      areaStyle: {
        color: {
          type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
          colorStops: [
            { offset: 0, color: pick('rgba(74, 222, 128, 0.25)', 'rgba(22, 163, 74, 0.2)') },
            { offset: 1, color: pick('rgba(74, 222, 128, 0.02)', 'rgba(22, 163, 74, 0.02)') },
          ],
        },
      },
    }],
  })
</script>

<BaseChart {option} height="240px" />
