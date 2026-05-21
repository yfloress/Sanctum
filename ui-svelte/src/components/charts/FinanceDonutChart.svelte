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

  interface Props {
    data: { name: string; value: number }[]
  }

  let { data }: Props = $props()

  const COLORS = [
    '#a855f7', '#4ade80', '#fbbf24', '#f87171',
    '#a78bfa', '#f472b6', '#34d399', '#fb923c', '#60a5fa',
  ]

  let option = $derived({
    backgroundColor: 'transparent',
    tooltip: {
      trigger: 'item',
      backgroundColor: pick('#1a1a1a', L.tooltipBg),
      borderColor: pick('#333', L.tooltipBorder),
      textStyle: { color: pick('#e0e0e0', L.tooltipText), fontSize: 12 },
      formatter: (p: { name: string; percent: number }) =>
        `${p.name}: ${p.percent.toFixed(1)}%`,
    },
    legend: {
      orient: 'vertical',
      right: 8,
      top: 'center',
      textStyle: { color: pick('#555', L.labelDim), fontSize: 11 },
      icon: 'circle',
      itemWidth: 8,
      itemHeight: 8,
    },
    series: [{
      type: 'pie',
      radius: ['40%', '68%'],
      center: ['38%', '50%'],
      avoidLabelOverlap: true,
      itemStyle: { borderColor: pick('#0d0d12', L.sliceBorder), borderWidth: 2, borderRadius: 4 },
      label: { show: false },
      emphasis: {
        label: { show: true, fontSize: 12, fontWeight: 'bold', color: pick('#e8eaed', L.centerLabel) },
      },
      data: data.map((d, i) => ({
        name: d.name,
        value: d.value,
        itemStyle: { color: COLORS[i % COLORS.length] },
      })),
    }],
  })
</script>

<BaseChart {option} height="220px" />
