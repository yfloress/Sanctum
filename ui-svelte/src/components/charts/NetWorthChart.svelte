<script lang="ts">
  import BaseChart from './BaseChart.svelte'
  import type { NetWorthChartData } from '../../lib/types'

  interface Props {
    data: NetWorthChartData
  }

  let { data }: Props = $props()

  let option = $derived({
    backgroundColor: 'transparent',
    grid: { left: 60, right: 20, top: 20, bottom: 30 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: '#1a1a1a',
      borderColor: '#333',
      textStyle: { color: '#e0e0e0', fontSize: 12 },
    },
    xAxis: {
      type: 'category',
      data: data.dates,
      axisLine: { lineStyle: { color: '#333' } },
      axisLabel: { color: '#666', fontSize: 10 },
    },
    yAxis: {
      type: 'value',
      axisLine: { show: false },
      splitLine: { lineStyle: { color: '#1a1a1a' } },
      axisLabel: { color: '#666', fontSize: 10 },
    },
    series: [{
      type: 'line',
      data: data.values,
      smooth: true,
      showSymbol: false,
      lineStyle: { color: '#4f9cf7', width: 2 },
      areaStyle: {
        color: {
          type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(79, 156, 247, 0.3)' },
            { offset: 1, color: 'rgba(79, 156, 247, 0.02)' },
          ],
        },
      },
    }],
  })
</script>

<BaseChart {option} height="260px" />
