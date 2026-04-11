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
  import * as echarts from 'echarts/core'
  import { CanvasRenderer } from 'echarts/renderers'
  import {
    LineChart, BarChart, PieChart, RadarChart,
  } from 'echarts/charts'
  import {
    GridComponent, TooltipComponent, LegendComponent,
    RadarComponent,
  } from 'echarts/components'

  echarts.use([
    CanvasRenderer, LineChart, BarChart, PieChart, RadarChart,
    GridComponent, TooltipComponent, LegendComponent, RadarComponent,
  ])

  interface Props {
    option: echarts.EChartsCoreOption
    height?: string
  }

  let { option, height = '280px' }: Props = $props()

  let container: HTMLDivElement
  let chart: echarts.ECharts | null = null

  $effect(() => {
    if (!container) return

    if (!chart) {
      chart = echarts.init(container, 'dark')
    }

    chart.setOption(option, { notMerge: true })

    const ro = new ResizeObserver(() => chart?.resize())
    ro.observe(container)

    return () => {
      ro.disconnect()
      chart?.dispose()
      chart = null
    }
  })
</script>

<div bind:this={container} style="width: 100%; height: {height}"></div>
