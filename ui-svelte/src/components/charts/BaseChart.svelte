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
