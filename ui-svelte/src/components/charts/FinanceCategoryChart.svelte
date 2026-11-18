<!-- Sanctum — a privacy-first personal finance and crypto vault.
     Copyright (C) 2026  yfloress

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

  interface Props {
    data: { name: string; value: number }[]
  }

  let { data }: Props = $props()

  let option = $derived({
    backgroundColor: 'transparent',
    grid: { left: 110, right: 24, top: 8, bottom: 24 },
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
      backgroundColor: pick('#18161f', L.tooltipBg),
      borderColor: pick('#2f2c35', L.tooltipBorder),
      textStyle: { color: pick('#eeeef0', L.tooltipText), fontSize: 12 },
    },
    xAxis: {
      type: 'value',
      axisLine: { show: false },
      axisTick: { show: false },
      splitLine: { lineStyle: { color: pick('#1a1820', L.splitLine) } },
      axisLabel: { color: pick('#7c7a85', L.labelDim), fontSize: 10 },
    },
    yAxis: {
      type: 'category',
      data: data.map(d => d.name),
      axisLine: { lineStyle: { color: pick('#2b2930', L.axisLine) } },
      axisTick: { show: false },
      axisLabel: { color: pick('#8e8c99', L.label), fontSize: 11 },
    },
    series: [{
      type: 'bar',
      data: data.map(d => d.value),
      barMaxWidth: 18,
      itemStyle: {
        borderRadius: [0, 4, 4, 0],
        color: {
          type: 'linear', x: 0, y: 0, x2: 1, y2: 0,
          colorStops: [
            { offset: 0, color: pick('#f87171', L.negative) },
            { offset: 1, color: pick('#a855f7', L.accent) },
          ],
        },
      },
    }],
  })
</script>

<BaseChart {option} height="260px" />
