<script lang="ts">
  import BaseChart from './BaseChart.svelte'
  import type { DistributionItem } from '../../lib/types'

  interface Props {
    data: DistributionItem[]
  }

  let { data }: Props = $props()

  const colors = ['#4f9cf7', '#4ade80', '#fbbf24', '#f87171', '#a78bfa', '#f472b6', '#34d399', '#fb923c', '#60a5fa', '#e879f9']

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
