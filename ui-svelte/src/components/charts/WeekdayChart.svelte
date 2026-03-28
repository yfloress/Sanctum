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
      itemStyle: { color: '#4f9cf7', borderRadius: [4, 4, 0, 0] },
    }],
  })
</script>

<BaseChart {option} height="200px" />
