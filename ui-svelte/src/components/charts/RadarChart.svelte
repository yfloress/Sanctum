<script lang="ts">
  import BaseChart from './BaseChart.svelte'
  import type { RadarChartData } from '../../lib/types'

  interface Props {
    data: RadarChartData
  }

  let { data }: Props = $props()

  let option = $derived({
    backgroundColor: 'transparent',
    radar: {
      indicator: data.categories.map(c => ({ name: c, max: data.max_value })),
      shape: 'polygon',
      axisName: { color: '#888', fontSize: 11 },
      splitLine: { lineStyle: { color: '#222' } },
      splitArea: { areaStyle: { color: ['transparent'] } },
      axisLine: { lineStyle: { color: '#333' } },
    },
    series: [{
      type: 'radar',
      data: [{
        value: data.values,
        areaStyle: { color: 'rgba(168, 85, 247, 0.2)' },
        lineStyle: { color: '#a855f7', width: 2 },
        itemStyle: { color: '#a855f7' },
      }],
    }],
  })
</script>

<BaseChart {option} height="280px" />
