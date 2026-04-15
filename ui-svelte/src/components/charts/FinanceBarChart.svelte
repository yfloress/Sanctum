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

  interface Props {
    months: string[]
    income: number[]
    expenses: number[]
  }

  let { months, income, expenses }: Props = $props()

  let option = $derived({
    backgroundColor: 'transparent',
    grid: { left: 52, right: 12, top: 16, bottom: 36 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: '#1a1a1a',
      borderColor: '#333',
      textStyle: { color: '#e0e0e0', fontSize: 12 },
    },
    legend: {
      data: ['Income', 'Expenses'],
      bottom: 0,
      textStyle: { color: '#555', fontSize: 11 },
      icon: 'roundRect',
      itemWidth: 10,
      itemHeight: 10,
    },
    xAxis: {
      type: 'category',
      data: months,
      axisLine: { lineStyle: { color: '#2a2a2a' } },
      axisTick: { show: false },
      axisLabel: { color: '#555', fontSize: 11 },
    },
    yAxis: {
      type: 'value',
      axisLine: { show: false },
      axisTick: { show: false },
      splitLine: { lineStyle: { color: '#1e1e1e' } },
      axisLabel: { color: '#555', fontSize: 10 },
    },
    series: [
      {
        name: 'Income',
        type: 'bar',
        data: income,
        barMaxWidth: 20,
        itemStyle: { color: '#4ade80', borderRadius: [4, 4, 0, 0] },
      },
      {
        name: 'Expenses',
        type: 'bar',
        data: expenses,
        barMaxWidth: 20,
        itemStyle: { color: '#f87171', borderRadius: [4, 4, 0, 0] },
      },
    ],
  })
</script>

<BaseChart {option} height="220px" />
