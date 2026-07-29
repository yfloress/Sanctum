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
  import { i18n } from '../../lib/stores/i18n.svelte'

  interface Props {
    months: string[]
    income: number[]
    expenses: number[]
  }

  let { months, income, expenses }: Props = $props()

  let incomeLabel = $derived(i18n.t('finances-income', 'Income'))
  let expensesLabel = $derived(i18n.t('finances-expenses', 'Expenses'))

  let option = $derived({
    backgroundColor: 'transparent',
    grid: { left: 52, right: 12, top: 16, bottom: 36 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: pick('#1a1a1a', L.tooltipBg),
      borderColor: pick('#333', L.tooltipBorder),
      textStyle: { color: pick('#e0e0e0', L.tooltipText), fontSize: 12 },
    },
    legend: {
      data: [incomeLabel, expensesLabel],
      bottom: 0,
      textStyle: { color: pick('#555', L.labelDim), fontSize: 11 },
      icon: 'roundRect',
      itemWidth: 10,
      itemHeight: 10,
    },
    xAxis: {
      type: 'category',
      data: months,
      axisLine: { lineStyle: { color: pick('#2a2a2a', L.axisLine) } },
      axisTick: { show: false },
      axisLabel: { color: pick('#555', L.labelDim), fontSize: 11 },
    },
    yAxis: {
      type: 'value',
      axisLine: { show: false },
      axisTick: { show: false },
      splitLine: { lineStyle: { color: pick('#1e1e1e', L.splitLine) } },
      axisLabel: { color: pick('#555', L.labelDim), fontSize: 10 },
    },
    series: [
      {
        name: incomeLabel,
        type: 'bar',
        data: income,
        barMaxWidth: 20,
        itemStyle: { color: pick('#4ade80', L.positive), borderRadius: [4, 4, 0, 0] },
      },
      {
        name: expensesLabel,
        type: 'bar',
        data: expenses,
        barMaxWidth: 20,
        itemStyle: { color: pick('#f87171', L.negative), borderRadius: [4, 4, 0, 0] },
      },
    ],
  })
</script>

<BaseChart {option} height="220px" />
