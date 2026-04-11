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
  import type { DistributionItem } from '../../lib/types'

  interface Props {
    data: DistributionItem[]
  }

  let { data }: Props = $props()

  const colors = ['#a855f7', '#4ade80', '#fbbf24', '#f87171', '#a78bfa', '#f472b6', '#34d399', '#fb923c', '#60a5fa', '#e879f9']

  let option = $derived({
    backgroundColor: 'transparent',
    tooltip: {
      trigger: 'item',
      backgroundColor: '#1a1a1a',
      borderColor: '#333',
      textStyle: { color: '#e0e0e0', fontSize: 12 },
      formatter: (p: { name: string, percent: number, value: number }) =>
        `${p.name}: $${p.value.toFixed(2)} (${p.percent}%)`,
    },
    series: [{
      type: 'pie',
      radius: ['45%', '75%'],
      center: ['50%', '50%'],
      avoidLabelOverlap: true,
      itemStyle: { borderColor: '#111', borderWidth: 2, borderRadius: 4 },
      label: {
        color: '#ccc',
        fontSize: 11,
        formatter: '{b} {d}%',
      },
      data: data.map((d, i) => ({
        name: d.symbol,
        value: d.value,
        itemStyle: { color: colors[i % colors.length] },
      })),
    }],
  })
</script>

<BaseChart {option} height="300px" />
