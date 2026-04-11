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
  import type { WeekdayChartData } from '../../lib/types'

  interface Props {
    data: WeekdayChartData
  }

  let { data }: Props = $props()

  let option = $derived({
    backgroundColor: 'transparent',
    grid: { left: 40, right: 20, top: 10, bottom: 30 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: '#1a1a1a',
      borderColor: '#333',
      textStyle: { color: '#e0e0e0', fontSize: 12 },
      formatter: (p: { name: string, value: number }[]) =>
        `${p[0].name}: ${(p[0].value * 100).toFixed(0)}%`,
    },
    xAxis: {
      type: 'category',
      data: data.labels,
      axisLine: { lineStyle: { color: '#333' } },
      axisLabel: { color: '#888', fontSize: 11 },
    },
    yAxis: {
      type: 'value',
      max: 1,
      axisLine: { show: false },
      splitLine: { lineStyle: { color: '#1a1a1a' } },
      axisLabel: {
        color: '#666', fontSize: 10,
        formatter: (v: number) => `${(v * 100).toFixed(0)}%`,
      },
    },
    series: [{
      type: 'bar',
      data: data.values,
      barWidth: '50%',
      itemStyle: { color: '#a855f7', borderRadius: [4, 4, 0, 0] },
    }],
  })
</script>

<BaseChart {option} height="200px" />
