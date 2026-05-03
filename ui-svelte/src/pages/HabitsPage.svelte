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
  import { app } from '../lib/stores/app.svelte'
  import { i18n } from '../lib/stores/i18n.svelte'
  import * as habitsApi from '../lib/api/habits'
  import RadarChart from '../components/charts/RadarChart.svelte'
  import WeekdayChart from '../components/charts/WeekdayChart.svelte'
  import HabitFormModal from '../components/habits/HabitFormModal.svelte'
  import HabitHeatmap from '../components/habits/HabitHeatmap.svelte'
  import HabitRewardsPanel from '../components/habits/HabitRewardsPanel.svelte'
  import HabitGoalsPanel from '../components/habits/HabitGoalsPanel.svelte'
  import type {
    HabitDto, HabitsResponse, HabitSummary,
    HeatmapResponse, HabitAnalyticsResponse,
    StreakRewardDto, GoalDto, AchievementDto
  } from '../lib/types'

  type Tab = 'habits' | 'rewards' | 'history'
  let activeTab = $state<Tab>('habits')
  let loading = $state(true)

  // Habits tab state
  let habitsData = $state<HabitsResponse | null>(null)
  let selectedHabit = $state<HabitDto | null>(null)
  let summary = $state<HabitSummary | null>(null)
  let heatmap = $state<HeatmapResponse | null>(null)
  let analytics = $state<HabitAnalyticsResponse | null>(null)
  let heatmapYear = $state(new Date().getFullYear())

  // Rewards tab state
  let rewards = $state<StreakRewardDto[]>([])
  let goals = $state<GoalDto[]>([])

  // History tab state
  let achievements = $state<AchievementDto[]>([])

  // Month navigation
  let month = $state(new Date().getMonth() + 1)
  let year = $state(new Date().getFullYear())

  // Modal state
  let showAddHabit = $state(false)
  let editingHabit = $state<HabitDto | null>(null)

  async function load() {
    loading = true
    try {
      habitsData = await habitsApi.fetchHabits(month, year)
      heatmap = await habitsApi.fetchHeatmap(heatmapYear)
      analytics = await habitsApi.fetchHabitAnalytics()
    } catch (e) {
      app.showToast(String(e), true)
    } finally {
      loading = false
    }
  }

  async function loadRewards() {
    try {
      const [r, g] = await Promise.all([
        habitsApi.fetchRewards(),
        habitsApi.fetchGoals(),
      ])
      rewards = r
      goals = g
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function loadHistory() {
    try {
      achievements = await habitsApi.fetchAchievements()
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function prevMonth() {
    if (month === 1) { month = 12; year-- }
    else { month-- }
    load()
  }

  function nextMonth() {
    if (month === 12) { month = 1; year++ }
    else { month++ }
    load()
  }

  async function toggleDay(habitId: string, day: number) {
    const dateStr = `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`
    try {
      await habitsApi.toggleHabit(habitId, dateStr)
      const [newHabits, newHeatmap, newAnalytics] = await Promise.all([
        habitsApi.fetchHabits(month, year),
        habitsApi.fetchHeatmap(heatmapYear),
        habitsApi.fetchHabitAnalytics(),
      ])
      habitsData = newHabits
      heatmap = newHeatmap
      analytics = newAnalytics
      if (selectedHabit?.id === habitId) {
        summary = await habitsApi.fetchHabitSummary(habitId)
      }
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function selectHabit(habit: HabitDto) {
    selectedHabit = habit
    try {
      summary = await habitsApi.fetchHabitSummary(habit.id)
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function openAddHabit() {
    editingHabit = null
    showAddHabit = true
  }

  function openEditHabit(h: HabitDto) {
    editingHabit = h
    showAddHabit = true
  }

  async function deleteHabit(id: string) {
    try {
      await habitsApi.deleteHabit(id)
      if (selectedHabit?.id === id) { selectedHabit = null; summary = null }
      await load()
      app.showToast(i18n.t('habits-toast-habit-deleted', 'Habit deleted'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  const monthNames = $derived([
    i18n.t('month-january','January'), i18n.t('month-february','February'), i18n.t('month-march','March'),
    i18n.t('month-april','April'), i18n.t('month-may','May'), i18n.t('month-june','June'),
    i18n.t('month-july','July'), i18n.t('month-august','August'), i18n.t('month-september','September'),
    i18n.t('month-october','October'), i18n.t('month-november','November'), i18n.t('month-december','December'),
  ])

  const WEEKDAYS = ['S', 'M', 'T', 'W', 'T', 'F', 'S'] as const

  const now = new Date()
  const viewingCurrentMonth = $derived(now.getFullYear() === year && now.getMonth() + 1 === month)
  const todayDay = $derived(viewingCurrentMonth ? now.getDate() : -1)

  function weekdayOf(day: number): number {
    return new Date(year, month - 1, day).getDay()
  }

  function trailingStreak(habit: HabitDto, daysInMonth: number): number {
    const lastDay = viewingCurrentMonth ? todayDay : daysInMonth
    let s = 0
    for (let i = lastDay; i >= 1; i--) {
      if (habit.days[i]) s++
      else break
    }
    return s
  }

  $effect(() => { load() })
  $effect(() => { if (activeTab === 'rewards') loadRewards() })
  $effect(() => { if (activeTab === 'history') loadHistory() })
</script>

<div class="page"   class:blurred={showAddHabit}>
  <div class="page-header">
    <h2>{i18n.t('habits-title', 'HABITS')}</h2>
    {#if activeTab === 'habits'}
      <div class="month-nav">
        <button class="nav-arrow" aria-label="Previous month" onclick={prevMonth}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 19l-7-7 7-7"/></svg>
        </button>
        <span class="month-label">{monthNames[month - 1]} {year}</span>
        <button class="nav-arrow" aria-label="Next month" onclick={nextMonth}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 5l7 7-7 7"/></svg>
        </button>
      </div>
    {/if}
  </div>

  <div class="tab-bar">
    <button class:active={activeTab === 'habits'} onclick={() => activeTab = 'habits'}>{i18n.t('habits-tab-habits', 'Habits')}</button>
    <button class:active={activeTab === 'rewards'} onclick={() => activeTab = 'rewards'}>{i18n.t('habits-tab-rewards', 'Rewards')}</button>
    <button class:active={activeTab === 'history'} onclick={() => activeTab = 'history'}>{i18n.t('habits-tab-history', 'History')}</button>
  </div>

  {#if loading}
    <div class="skeleton-page">
      <div class="skeleton" style="width:100%;height:110px;border-radius:var(--radius-lg);margin-bottom:16px"></div>
      <div class="skeleton" style="width:100%;height:80px;border-radius:var(--radius-lg);margin-bottom:16px"></div>
      <div class="skeleton-row">
        <div class="skeleton" style="flex:1;height:160px;border-radius:var(--radius-lg)"></div>
        <div class="skeleton" style="flex:1;height:160px;border-radius:var(--radius-lg)"></div>
      </div>
    </div>

  <!-- HABITS TAB -->
  {:else if activeTab === 'habits'}
    <div class="section-header">
      <h3>{i18n.t('habits-daily-tracking', 'Daily Tracking')}</h3>
      <button class="glass-btn" onclick={openAddHabit}>{i18n.t('habits-new-habit', 'New Habit')}</button>
    </div>

    {#if habitsData && habitsData.habits.length > 0}
      <div class="habit-grid">
        <div class="habit-grid-scroll">
          <div class="grid-header">
            <span class="habit-name-col weekday-spacer"></span>
            {#each Array(habitsData.days_in_month) as _, i}
              {@const wd = weekdayOf(i + 1)}
              <span class="weekday" class:weekend={wd === 0 || wd === 6}>{WEEKDAYS[wd]}</span>
            {/each}
          </div>
          <div class="grid-header">
            <span class="habit-name-col">{i18n.t('habits-habit-col', 'Habit')}</span>
            {#each Array(habitsData.days_in_month) as _, i}
              <span class="day-num" class:is-today={i + 1 === todayDay}>{i + 1}</span>
            {/each}
          </div>
          {#each habitsData.habits as habit}
            {@const streak = trailingStreak(habit, habitsData.days_in_month)}
            <div class="grid-row">
              <button
                class="habit-name-col clickable"
                onclick={() => selectHabit(habit)}
                style="border-left: 3px solid {habit.color}"
              >
                <span class="habit-name-text">{habit.name}</span>
                {#if streak > 0}
                  <span class="streak-pill" style="color: {habit.color}; border-color: {habit.color}40">
                    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 2s4 4 4 8a4 4 0 01-8 0c0-1 .3-2 .8-3C10 8 10 6 12 2zm0 11a3 3 0 110 6 3 3 0 010-6z"/></svg>
                    {streak}
                  </span>
                {/if}
              </button>
              {#each habit.days.slice(1, habitsData.days_in_month + 1) as done, i}
                <button
                  class="day-cell"
                  class:done
                  class:is-today={i + 1 === todayDay}
                  style={done ? `background: ${habit.color}; border-color: ${habit.color}` : ''}
                  onclick={() => toggleDay(habit.id, i + 1)}
                  aria-label="Day {i + 1}"
                ></button>
              {/each}
            </div>
          {/each}
        </div>
      </div>

      <!-- Activity Heatmap -->
      {#if heatmap}
        <HabitHeatmap
          bind:heatmapYear={heatmapYear}
          heatmap={heatmap}
          onyearchange={async (year) => { heatmapYear = year; try { heatmap = await habitsApi.fetchHeatmap(heatmapYear) } catch (e) { app.showToast(String(e), true) } }}
        />
      {/if}

      <!-- Selected habit summary -->
      {#if selectedHabit && summary}
        <div class="summary-card">
          <div class="summary-header">
            <h3 style="border-left: 3px solid {selectedHabit.color}; padding-left: 8px">
              {selectedHabit.name}
            </h3>
            <div class="summary-actions">
              <button class="icon-btn" onclick={() => openEditHabit(selectedHabit!)}>{i18n.t('habits-edit', 'Edit')}</button>
              <button class="icon-btn danger" onclick={() => deleteHabit(selectedHabit!.id)}>{i18n.t('habits-delete', 'Delete')}</button>
            </div>
          </div>
          <div class="stats-grid">
            <div class="stat">
              <svg class="stat-icon flame" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 2s4 4 4 8a4 4 0 01-8 0c0-1 .3-2 .8-3C10 8 10 6 12 2zm0 11a3 3 0 110 6 3 3 0 010-6z"/></svg>
              <span class="stat-val">{summary.current_streak}</span>
              <span class="stat-lbl">{i18n.t('habits-current-streak', 'Current Streak')}</span>
            </div>
            <div class="stat">
              <svg class="stat-icon trophy" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M6 3h12v4a6 6 0 01-12 0V3zM4 5h2M18 5h2M12 13v4M8 21h8M10 17h4"/></svg>
              <span class="stat-val">{summary.best_streak}</span>
              <span class="stat-lbl">{i18n.t('habits-best-streak', 'Best Streak')}</span>
            </div>
            <div class="stat">
              <svg class="stat-icon pct" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M8 15l8-8M9 9h.01M15 15h.01"/></svg>
              <span class="stat-val">{(summary.completion_rate * 100).toFixed(0)}%</span>
              <span class="stat-lbl">{i18n.t('habits-completion', 'Completion')}</span>
            </div>
            <div class="stat">
              <svg class="stat-icon cal" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="3" y="5" width="18" height="16" rx="2"/><path d="M3 10h18M8 3v4M16 3v4"/></svg>
              <span class="stat-val">{summary.last_30_days}</span>
              <span class="stat-lbl">{i18n.t('habits-last-30-days', 'Last 30 Days')}</span>
            </div>
          </div>
        </div>
      {/if}

      <!-- Analytics -->
      {#if analytics}
        <div class="analytics-section">
          {#if analytics.radar.categories.length > 0}
            <div class="chart-card">
              <h3>{i18n.t('habits-habit-radar', 'Habit Radar')}</h3>
              <RadarChart data={analytics.radar} />
            </div>
          {/if}
          {#if analytics.weekday_efficiency.labels.length > 0}
            <div class="chart-card">
              <h3>{i18n.t('habits-weekday-efficiency', 'Weekday Efficiency')}</h3>
              <WeekdayChart data={analytics.weekday_efficiency} />
            </div>
          {/if}
          <div class="analytics-card">
            <p class="insight">{analytics.weekly_summary}</p>
            <p class="insight">{analytics.insight}</p>
          </div>
        </div>
      {/if}
    {:else}
      <p class="empty">{i18n.t('habits-no-habits', 'No habits yet. Create your first habit to start tracking.')}</p>
    {/if}

  <!-- REWARDS TAB -->
  {:else if activeTab === 'rewards'}
    <div class="rewards-section">
      <HabitRewardsPanel
        rewards={rewards}
        habits={habitsData?.habits ?? []}
        onrefresh={loadRewards}
      />

      <HabitGoalsPanel
        goals={goals}
        onrefresh={loadRewards}
        ongoalsupdate={(updated) => goals = updated}
      />
    </div>

  <!-- HISTORY TAB -->
  {:else if activeTab === 'history'}
    {#if achievements.length === 0}
      <p class="empty">{i18n.t('habits-no-achievements', 'No achievements yet.')}</p>
    {:else}
      <div class="timeline">
        {#each achievements as ach}
          <div class="timeline-item">
            <div class="timeline-dot"></div>
            <div class="timeline-content">
              <span class="ach-title">{ach.title}</span>
              <span class="ach-desc">{ach.description}</span>
              <span class="ach-date">{ach.achieved_at}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<!-- Add/Edit Habit Modal -->
<HabitFormModal
  bind:show={showAddHabit}
  editing={editingHabit}
  onsubmit={load}
  onclose={() => showAddHabit = false}
/>

<style>
  .page { padding: 24px 32px; max-width: 1000px; width: 100%; margin: 0 auto; }

  .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  h2 { font-size: 1.3rem; letter-spacing: 0.2em; color: var(--text-primary); margin: 0; }

  .month-nav { display: flex; align-items: center; gap: 12px; }
  .nav-arrow { background: none; border: none; color: var(--text-secondary); cursor: pointer; padding: 4px; display: flex; transition: color 0.15s; }
  .nav-arrow:hover { color: var(--text-primary); }
  .nav-arrow svg { width: 18px; height: 18px; }
  .month-label { font-size: 0.9rem; color: var(--text-secondary); min-width: 140px; text-align: center; }

  .skeleton-page { padding: 8px 0; }
  .empty { text-align: center; padding: 48px; color: var(--text-tertiary); }

  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .section-header h3 { font-size: 0.85rem; color: var(--text-secondary); text-transform: uppercase; margin: 0; }

  /* Habit grid */
  .habit-grid {
    position: relative;
    overflow: hidden; margin-bottom: 24px;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    box-shadow: var(--card-shadow);
  }
  .habit-grid::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.5;
    z-index: 2;
  }
  .habit-grid::after {
    content: '';
    position: absolute;
    top: 0; right: 0; bottom: 0;
    width: 56px;
    background: linear-gradient(to right, transparent, var(--card-bg-solid, var(--card-bg)));
    pointer-events: none;
    z-index: 1;
    border-radius: 0 var(--radius-lg) var(--radius-lg) 0;
  }
  .habit-grid-scroll {
    overflow-x: auto;
    padding: 14px 16px;
    scrollbar-width: none;
  }
  .habit-grid-scroll::-webkit-scrollbar { display: none; }
  .grid-header, .grid-row { display: flex; align-items: center; gap: 2px; min-width: max-content; }
  .grid-header { margin-bottom: 2px; }
  .grid-row { padding: 2px 0; }
  .habit-name-col {
    width: 140px; flex-shrink: 0; font-size: 0.8rem; color: var(--text-secondary);
    padding: 4px 8px; overflow: hidden; white-space: nowrap;
    display: flex; align-items: center; justify-content: space-between; gap: 6px;
  }
  .habit-name-col.weekday-spacer { padding: 0; }
  .habit-name-text { overflow: hidden; text-overflow: ellipsis; }
  .habit-name-col.clickable {
    background: none; border: none; cursor: pointer; color: var(--text-secondary);
    text-align: left; border-radius: 4px; transition: background 0.15s, color 0.15s;
  }
  .habit-name-col.clickable:hover { background: var(--glass-hover); color: var(--text-primary); }

  .streak-pill {
    display: inline-flex; align-items: center; gap: 3px; flex-shrink: 0;
    padding: 1px 7px 1px 4px; border-radius: 999px;
    border: 1px solid currentColor;
    font-size: 0.68rem; font-weight: 600;
    background: rgba(255, 255, 255, 0.03);
  }
  .streak-pill svg { width: 10px; height: 10px; }

  .weekday {
    width: 18px; height: 14px; display: flex; align-items: center; justify-content: center;
    font-size: 0.55rem; font-weight: 600; color: var(--text-tertiary);
    text-transform: uppercase; letter-spacing: 0.05em;
  }
  .weekday.weekend { color: var(--accent); opacity: 0.55; }

  .day-num {
    width: 18px; height: 16px; display: flex; align-items: center; justify-content: center;
    font-size: 0.6rem; color: var(--text-tertiary); position: relative;
  }
  .day-num.is-today { color: var(--accent); font-weight: 700; }
  .day-num.is-today::after {
    content: ''; position: absolute; bottom: -2px; left: 50%; transform: translateX(-50%);
    width: 3px; height: 3px; border-radius: 50%; background: var(--accent);
  }

  .day-cell {
    width: 18px; height: 18px; border-radius: 3px; border: 1px solid var(--glass-border);
    background: var(--glass); cursor: pointer; padding: 0;
    transition: transform 0.12s ease, border-color 0.15s, background 0.15s;
  }
  .day-cell:hover {
    border-color: var(--glass-border-hover); background: var(--glass-hover);
    transform: scale(1.2); z-index: 1;
  }
  .day-cell.done { box-shadow: 0 0 6px rgba(255,255,255,0.08); }
  .day-cell.done:hover { transform: scale(1.25); }
  .day-cell.is-today:not(.done) { box-shadow: 0 0 0 1px var(--accent) inset; }

  /* Summary card */
  .summary-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 20px; margin-bottom: 20px; box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .summary-card::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.5;
  }
  .summary-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .summary-header h3 { margin: 0; font-size: 1rem; color: var(--text-primary); }
  .summary-actions { display: flex; gap: 8px; }
  .icon-btn {
    background: none; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    color: var(--text-secondary); cursor: pointer; padding: 4px 12px; font-size: 0.8rem;
    transition: all 0.15s;
  }
  .icon-btn:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }
  .icon-btn.danger:hover { color: var(--danger); border-color: var(--danger-border); background: var(--danger-bg); }

  .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; }
  .stat {
    display: flex; flex-direction: column; align-items: center; gap: 2px;
    padding: 10px 8px; border-radius: var(--radius-sm);
    background: var(--glass-elevated);
    border: 1px solid var(--glass-border);
    transition: border-color 0.2s, box-shadow 0.2s;
  }
  .stat:hover { border-color: var(--glass-border-hover); box-shadow: var(--glass-shadow); }
  .stat-icon { width: 16px; height: 16px; color: var(--text-tertiary); margin-bottom: 2px; }
  .stat-icon.flame { color: #fb923c; }
  .stat-icon.trophy { color: var(--warning); }
  .stat-icon.pct { color: var(--accent); }
  .stat-icon.cal { color: var(--success); }
  .stat-val { font-size: 1.35rem; font-weight: 700; color: var(--text-primary); line-height: 1.1; }
  .stat-lbl { font-size: 0.68rem; color: var(--text-tertiary); margin-top: 2px; text-align: center; }

  .analytics-section { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 24px; }
  .chart-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 16px; box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .chart-card::before, .analytics-card::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.5;
  }
  .chart-card h3 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin: 0 0 8px; }
  .analytics-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 16px; grid-column: 1 / -1; box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .insight { font-size: 0.85rem; color: var(--text-secondary); margin: 4px 0; }

  /* Timeline */
  .timeline { display: flex; flex-direction: column; gap: 0; padding-left: 20px; }
  .timeline-item { display: flex; gap: 16px; padding: 12px 0; border-left: 2px solid var(--glass-border); padding-left: 16px; position: relative; }
  .timeline-dot {
    position: absolute; left: -7px; top: 16px; width: 12px; height: 12px; border-radius: 50%;
    background: var(--accent); border: 2px solid var(--bg-base);
    box-shadow: 0 0 8px var(--accent-glow);
  }
  .timeline-content { display: flex; flex-direction: column; gap: 2px; }
  .ach-title { font-weight: 600; color: var(--text-primary); font-size: 0.9rem; }
  .ach-desc { font-size: 0.8rem; color: var(--text-secondary); }
  .ach-date { font-size: 0.7rem; color: var(--text-tertiary); }

</style>
