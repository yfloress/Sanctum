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
  import { app } from '../../lib/stores/app.svelte'

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
    const el = container
    if (!el) return

    const c = echarts.init(el, 'dark')
    chart = c

    const ro = new ResizeObserver(() => c.resize())
    ro.observe(el)

    return () => {
      ro.disconnect()
      c.dispose()
      chart = null
    }
  })

  $effect(() => {
    chart?.setOption(option)
  })
</script>

<div
  bind:this={container}
  class="chart-canvas"
  class:balances-hidden={app.hideBalances}
  style="width: 100%; height: {height}"
></div>

<style>
  .chart-canvas {
    transition: filter 0.2s ease;
  }
  .chart-canvas.balances-hidden {
    filter: blur(10px);
    pointer-events: none;
  }
</style>
