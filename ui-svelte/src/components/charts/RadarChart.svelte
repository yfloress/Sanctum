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
  import { chartLight as L, pick } from '../../lib/charts/theme'
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
      axisName: { color: pick('#888', L.label), fontSize: 11 },
      splitLine: { lineStyle: { color: pick('#222', L.splitLine) } },
      splitArea: { areaStyle: { color: ['transparent'] } },
      axisLine: { lineStyle: { color: pick('#333', L.axisLine) } },
    },
    series: [{
      type: 'radar',
      data: [{
        value: data.values,
        areaStyle: { color: pick('rgba(168, 85, 247, 0.2)', 'rgba(139, 92, 246, 0.18)') },
        lineStyle: { color: pick('#a855f7', L.accent), width: 2 },
        itemStyle: { color: pick('#a855f7', L.accent) },
      }],
    }],
  })
</script>

<BaseChart {option} height="280px" sensitive={false} />
