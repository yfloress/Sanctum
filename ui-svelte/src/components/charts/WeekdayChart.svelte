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
  import type { WeekdayChartData } from '../../lib/types'

  interface Props {
    data: WeekdayChartData
  }

  let { data }: Props = $props()

  let bestIdx = $derived.by(() => {
    let idx = -1
    let max = 0
    data.values.forEach((v, i) => {
      if (v > max) { max = v; idx = i }
    })
    return idx
  })

  let option = $derived({
    backgroundColor: 'transparent',
    grid: { left: 40, right: 20, top: 10, bottom: 30 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: pick('#1a1a1a', L.tooltipBg),
      borderColor: pick('#333', L.tooltipBorder),
      textStyle: { color: pick('#e0e0e0', L.tooltipText), fontSize: 12 },
      formatter: (p: { name: string, value: number }[]) =>
        `${p[0].name}: ${(p[0].value * 100).toFixed(0)}%`,
    },
    xAxis: {
      type: 'category',
      data: data.labels,
      axisLine: { lineStyle: { color: pick('#333', L.axisLine) } },
      axisLabel: { color: pick('#888', L.label), fontSize: 11 },
    },
    yAxis: {
      type: 'value',
      max: 1,
      axisLine: { show: false },
      splitLine: { lineStyle: { color: pick('#1a1a1a', L.splitLine) } },
      axisLabel: {
        color: pick('#666', L.labelDim), fontSize: 10,
        formatter: (v: number) => `${(v * 100).toFixed(0)}%`,
      },
    },
    series: [{
      type: 'bar',
      data: data.values.map((v, i) => ({
        value: v,
        itemStyle: {
          color: i === bestIdx ? pick('#4ade80', L.positive) : pick('#a855f7', L.accent),
          borderRadius: [4, 4, 0, 0],
          shadowBlur: i === bestIdx ? 12 : 0,
          shadowColor: i === bestIdx ? pick('rgba(74, 222, 128, 0.5)', 'rgba(22, 163, 74, 0.35)') : 'transparent',
        },
      })),
      barWidth: '50%',
    }],
  })
</script>

<BaseChart {option} height="200px" sensitive={false} />
