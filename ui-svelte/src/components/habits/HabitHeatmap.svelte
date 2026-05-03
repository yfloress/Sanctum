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
  import { i18n } from '../../lib/stores/i18n.svelte'
  import type { HeatmapResponse } from '../../lib/types'

  const MONTH_SHORT = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec']
  const todayStr = new Date().toISOString().slice(0, 10)

  interface HeatmapCell { week: number; weekday: number; date: string; intensity: number }

  interface Props {
    heatmap: HeatmapResponse
    heatmapYear: number
    onyearchange: (year: number) => Promise<void>
  }

  let { heatmap, heatmapYear = $bindable(new Date().getFullYear()), onyearchange }: Props = $props()

  const heatmapData = $derived.by(() => {
    if (!heatmap) return { cells: [] as HeatmapCell[], monthLabels: [] as { label: string; col: number }[], totalWeeks: 0 }
    const cells: HeatmapCell[] = []
    let week = 0
    heatmap.data.forEach((day, idx) => {
      const wd = new Date(day.date + 'T00:00:00').getDay()
      if (wd === 0 && idx > 0) week++
      cells.push({ week, weekday: wd, date: day.date, intensity: day.intensity })
    })

    const monthLabels: { label: string; col: number }[] = []
    let lastMonth = -1
    cells.forEach(c => {
      const m = new Date(c.date + 'T00:00:00').getMonth()
      if (m !== lastMonth) {
        const label = MONTH_SHORT[m]
        if (monthLabels.length === 0 || c.week - monthLabels[monthLabels.length - 1].col > 1) {
          monthLabels.push({ label, col: c.week })
        }
        lastMonth = m
      }
    })
    return { cells, monthLabels, totalWeeks: week + 1 }
  })

  async function prevYear() {
    heatmapYear--
    await onyearchange(heatmapYear)
  }

  async function nextYear() {
    heatmapYear++
    await onyearchange(heatmapYear)
  }
</script>

<div class="chart-section heatmap-section">
  <div class="heatmap-header">
    <h3>{i18n.t('habits-activity-heatmap', 'Activity Heatmap')}</h3>
    <div class="heatmap-nav">
      <button class="nav-arrow" aria-label={i18n.t('habits-prev-year', 'Previous year')} onclick={prevYear}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 19l-7-7 7-7"/></svg>
      </button>
      <span class="heatmap-year">{heatmapYear}</span>
      <button class="nav-arrow" aria-label="Next year" onclick={nextYear}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 5l7 7-7 7"/></svg>
      </button>
    </div>
  </div>
  <div class="heatmap-scroll">
    <div
      class="heatmap-grid"
    >
      <div class="heatmap-corner"></div>
      {#each heatmapData.monthLabels as ml}
        <span
          class="heatmap-month-label"
          style="grid-column: {ml.col + 2}; grid-row: 1;"
        >{ml.label}</span>
      {/each}
      {#each ['S','M','T','W','T','F','S'] as d, i}
        {#if i === 1 || i === 3 || i === 5}
          <span
            class="heatmap-dow-label"
            style="grid-row: {i + 2}; grid-column: 1;"
          >{d}</span>
        {/if}
      {/each}
      {#each heatmapData.cells as cell}
        <div
          class="heatmap-cell"
          class:l1={cell.intensity === 1}
          class:l2={cell.intensity === 2}
          class:l3={cell.intensity === 3}
          class:l4={cell.intensity === 4}
          class:today={cell.date === todayStr}
          style="grid-column: {cell.week + 2}; grid-row: {cell.weekday + 2};"
          title="{cell.date} · {cell.intensity} {i18n.t('habits-completions', 'completions')}"
        ></div>
      {/each}
    </div>
    <div class="heatmap-legend">
      <span class="heatmap-legend-label">{i18n.t('habits-less', 'Less')}</span>
      <div class="heatmap-cell"></div>
      <div class="heatmap-cell l1"></div>
      <div class="heatmap-cell l2"></div>
      <div class="heatmap-cell l3"></div>
      <div class="heatmap-cell l4"></div>
      <span class="heatmap-legend-label">{i18n.t('habits-more', 'More')}</span>
    </div>
  </div>
</div>

<style>
  .chart-section {
    position: relative;
    background: var(--card-bg); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 16px 20px; margin-bottom: 24px;
    box-shadow: var(--card-shadow); overflow: hidden;
  }
  .chart-section::before {
    content: ''; position: absolute;
    top: 0; left: 0; right: 0; height: 1px;
    background: var(--card-accent-line); opacity: 0.5;
  }
  .heatmap-section {
    position: relative;
  }
  .heatmap-section::after {
    content: '';
    position: absolute;
    top: 0; right: 0; bottom: 0;
    width: 48px;
    background: linear-gradient(to right, transparent, var(--card-bg-solid, var(--card-bg)));
    pointer-events: none;
  }
  .chart-section h3 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin: 0; }
  .heatmap-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
  .heatmap-nav { display: flex; align-items: center; gap: 8px; }
  .heatmap-year { font-size: 0.85rem; color: var(--text-secondary); min-width: 40px; text-align: center; }
  .nav-arrow { background: none; border: none; color: var(--text-secondary); cursor: pointer; padding: 4px; display: flex; transition: color 0.15s; }
  .nav-arrow:hover { color: var(--text-primary); }
  .nav-arrow svg { width: 14px; height: 14px; }

  .heatmap-scroll {
    overflow-x: auto;
    padding-bottom: 4px;
    scrollbar-width: none;
  }
  .heatmap-scroll::-webkit-scrollbar { display: none; }

  .heatmap-grid {
    display: grid;
    grid-template-rows: 14px repeat(7, 12px);
    gap: 3px;
    min-width: max-content;
  }
  .heatmap-corner { grid-column: 1; grid-row: 1; }
  .heatmap-month-label {
    font-size: 0.62rem; color: var(--text-tertiary);
    white-space: nowrap; line-height: 1;
    align-self: end;
  }
  .heatmap-dow-label {
    font-size: 0.6rem; color: var(--text-tertiary);
    display: flex; align-items: center; justify-content: flex-end;
    line-height: 1; padding-right: 4px;
  }
  .heatmap-cell {
    width: 12px; height: 12px; border-radius: 2px;
    background: var(--glass);
    border: 1px solid transparent;
    transition: transform 0.1s;
    cursor: default;
  }
  .heatmap-cell:hover { transform: scale(1.4); z-index: 1; border-color: var(--glass-border-hover); }
  .heatmap-cell.l1 { background: rgba(139, 92, 246, 0.35); }
  .heatmap-cell.l2 { background: rgba(139, 92, 246, 0.6); }
  .heatmap-cell.l3 { background: rgba(168, 85, 247, 0.85); }
  .heatmap-cell.l4 { background: #a855f7; box-shadow: 0 0 6px rgba(168, 85, 247, 0.5); }
  .heatmap-cell.today { box-shadow: 0 0 0 1.5px var(--accent) !important; }

  .heatmap-legend { display: flex; align-items: center; gap: 3px; justify-content: flex-end; margin-top: 4px; }
  .heatmap-legend-label { font-size: 0.6rem; color: var(--text-tertiary); }
</style>
