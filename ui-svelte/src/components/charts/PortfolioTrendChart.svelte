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

  let option = $derived({
    backgroundColor: 'transparent',
    grid: { left: 60, right: 20, top: 20, bottom: 30 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: '#1a1a1a',
      borderColor: '#333',
      textStyle: { color: '#e0e0e0', fontSize: 12 },
      formatter: (params: { value: number }[]) => {
        const pt = params[0]
        if (!pt) return ''
        return `<b>${formatCurrency(pt.value)}</b>`
      },
    },
    xAxis: {
      type: 'category',
      data: data.dates,
      axisLine: { lineStyle: { color: '#333' } },
      axisLabel: { color: '#666', fontSize: 10 },
    },
    yAxis: {
      type: 'value',
      axisLine: { show: false },
      splitLine: { lineStyle: { color: '#1a1a1a' } },
      axisLabel: { color: '#666', fontSize: 10, formatter: (val: number) => formatYAxis(val) },
    },
    series: [{
      type: 'line',
      data: data.values,
      step: 'end',
      showSymbol: data.values.length <= 3,
      symbolSize: 6,
      itemStyle: { color: '#4ade80' },
      lineStyle: { color: '#4ade80', width: 2 },
      areaStyle: {
        color: {
          type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(74, 222, 128, 0.25)' },
            { offset: 1, color: 'rgba(74, 222, 128, 0.02)' },
          ],
        },
      },
    }],
  })
</script>

<BaseChart {option} height="240px" />
