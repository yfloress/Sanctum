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
  let showAddReward = $state(false)
  let showAddGoal = $state(false)
  let editingReward = $state<StreakRewardDto | null>(null)
  let editingGoal = $state<GoalDto | null>(null)

  // Reward form
  let rewardHabitId = $state('')
  let rewardConsecutive = $state(true)
  let rewardTargetDays = $state('')
  let rewardTargetTotal = $state('')

  // Goal form
  let goalName = $state('')
  let goalDescription = $state('')
  let goalRewardText = $state('')
  let goalDeadline = $state('')

  // History tab state
  let achievements = $state<AchievementDto[]>([])

  // Month navigation
  let month = $state(new Date().getMonth() + 1)
  let year = $state(new Date().getFullYear())

  // Modal state
  let showAddHabit = $state(false)
  let editingHabit = $state<HabitDto | null>(null)
  let habitName = $state('')
  let habitDescription = $state('')
  let habitColor = $state('#a855f7')
  let habitCategory = $state('general')

  const colors = ['#a855f7', '#4ade80', '#f87171', '#fbbf24', '#a78bfa', '#f472b6', '#34d399', '#fb923c']

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
    habitName = ''
    habitDescription = ''
    habitColor = '#a855f7'
    habitCategory = 'general'
    showAddHabit = true
  }

  function openEditHabit(h: HabitDto) {
    editingHabit = h
    habitName = h.name
    habitDescription = h.description ?? ''
    habitColor = h.color
    habitCategory = h.category
    showAddHabit = true
  }

  async function submitHabit() {
    try {
      const desc = habitDescription || null
      if (editingHabit) {
        await habitsApi.updateHabit(editingHabit.id, habitName, desc, habitColor, habitCategory)
      } else {
        await habitsApi.createHabit(habitName, desc, habitColor, habitCategory)
      }
      showAddHabit = false
      await load()
      app.showToast(editingHabit ? i18n.t('habits-toast-habit-updated', 'Habit updated') : i18n.t('habits-toast-habit-created', 'Habit created'))
    } catch (e) {
      app.showToast(String(e), true)
    }
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

  function openAddReward() {
    editingReward = null
    rewardHabitId = habitsData?.habits[0]?.id ?? ''
    rewardConsecutive = true
    rewardTargetDays = ''
    rewardTargetTotal = ''
    showAddReward = true
  }

  function openEditReward(r: StreakRewardDto) {
    editingReward = r
    rewardHabitId = r.habit_id
    rewardConsecutive = r.is_consecutive
    rewardTargetDays = String(r.target_days ?? '')
    rewardTargetTotal = String(r.target_total ?? '')
    showAddReward = true
  }

  async function submitReward() {
    if (!rewardHabitId || !rewardTargetDays) {
      app.showToast(i18n.t('habits-toast-fill-required', 'Please fill required fields'), true)
      return
    }
    try {
      const isEditing = !!editingReward
      if (editingReward) {
        await habitsApi.updateStreakReward(
          editingReward.id,
          rewardHabitId,
          rewardConsecutive,
          parseInt(rewardTargetDays),
          rewardTargetTotal ? parseInt(rewardTargetTotal) : 0,
          editingReward.milestones.map(m => [m.target_days, m.reward_text] as [number, string])
        )
      } else {
        await habitsApi.createStreakReward(
          rewardHabitId,
          rewardConsecutive,
          parseInt(rewardTargetDays),
          rewardTargetTotal ? parseInt(rewardTargetTotal) : 0
        )
      }
      showAddReward = false
      editingReward = null
      await loadRewards()
      app.showToast(isEditing ? i18n.t('habits-toast-reward-updated', 'Reward updated') : i18n.t('habits-toast-reward-created', 'Reward created'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteReward(id: string) {
    try {
      await habitsApi.deleteStreakReward(id)
      await loadRewards()
      app.showToast(i18n.t('habits-toast-reward-deleted', 'Reward deleted'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function openAddGoal() {
    editingGoal = null
    goalName = ''
    goalDescription = ''
    goalRewardText = ''
    goalDeadline = ''
    showAddGoal = true
  }

  function openEditGoal(g: GoalDto) {
    editingGoal = g
    goalName = g.name
    goalDescription = g.description ?? ''
    goalRewardText = g.reward_text ?? ''
    goalDeadline = g.deadline ?? ''
    showAddGoal = true
  }

  async function submitGoal() {
    if (!goalName) {
      app.showToast(i18n.t('habits-toast-enter-goal-name', 'Please enter a goal name'), true)
      return
    }
    try {
      const isEditing = !!editingGoal
      if (editingGoal) {
        await habitsApi.updateGoal(editingGoal.id, goalName, goalDescription, goalRewardText, goalDeadline)
      } else {
        await habitsApi.createGoal(goalName, goalDescription, goalRewardText, goalDeadline)
      }
      showAddGoal = false
      editingGoal = null
      await loadRewards()
      app.showToast(isEditing ? i18n.t('habits-toast-goal-updated', 'Goal updated') : i18n.t('habits-toast-goal-created', 'Goal created'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteGoal(id: string) {
    try {
      await habitsApi.deleteGoal(id)
      await loadRewards()
      app.showToast(i18n.t('habits-toast-goal-deleted', 'Goal deleted'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function toggleCheckpoint(goalId: string, checkpointId: string) {
    try {
      await habitsApi.toggleCheckpoint(goalId, checkpointId)
      goals = await habitsApi.fetchGoals()
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function completeGoal(id: string) {
    try {
      await habitsApi.completeGoal(id)
      goals = await habitsApi.fetchGoals()
      app.showToast(i18n.t('habits-toast-goal-completed', 'Goal completed!'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function archiveGoal(id: string) {
    try {
      await habitsApi.archiveGoal(id)
      await loadRewards()
      app.showToast(i18n.t('habits-toast-goal-archived', 'Goal archived'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function prevHeatmapYear() {
    heatmapYear--
    try { heatmap = await habitsApi.fetchHeatmap(heatmapYear) } catch (e) { app.showToast(String(e), true) }
  }

  async function nextHeatmapYear() {
    heatmapYear++
    try { heatmap = await habitsApi.fetchHeatmap(heatmapYear) } catch (e) { app.showToast(String(e), true) }
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

  function goalProgress(g: GoalDto): number {
    if (g.checkpoints.length === 0) return g.is_completed ? 100 : 0
    const done = g.checkpoints.filter(c => c.completed).length
    return Math.round((done / g.checkpoints.length) * 100)
  }

  function rewardProgress(r: StreakRewardDto): number {
    const target = r.target_days ?? r.target_total ?? 0
    if (target <= 0) return 0
    return Math.min(100, Math.round((r.current_progress / target) * 100))
  }

  $effect(() => { load() })
  $effect(() => { if (activeTab === 'rewards') loadRewards() })
  $effect(() => { if (activeTab === 'history') loadHistory() })
</script>

<div class="page" class:blurred={showAddHabit || showAddReward || showAddGoal}>
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
    <div class="loading">{i18n.t('habits-loading', 'Loading...')}</div>

  <!-- HABITS TAB -->
  {:else if activeTab === 'habits'}
    <div class="section-header">
      <h3>{i18n.t('habits-daily-tracking', 'Daily Tracking')}</h3>
      <button class="glass-btn" onclick={openAddHabit}>{i18n.t('habits-new-habit', 'New Habit')}</button>
    </div>

    {#if habitsData && habitsData.habits.length > 0}
      <div class="habit-grid">
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

      <!-- Heatmap placeholder -->
      {#if heatmap}
        <div class="chart-section">
          <div class="heatmap-header">
            <h3>{i18n.t('habits-activity-heatmap', 'Activity Heatmap')}</h3>
            <div class="heatmap-nav">
              <button class="nav-arrow" aria-label={i18n.t('habits-prev-year', 'Previous year')} onclick={prevHeatmapYear}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 19l-7-7 7-7"/></svg>
              </button>
              <span class="heatmap-year">{heatmapYear}</span>
              <button class="nav-arrow" aria-label="Next year" onclick={nextHeatmapYear}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 5l7 7-7 7"/></svg>
              </button>
            </div>
          </div>
          <div class="heatmap">
            {#each heatmap.data as day}
              <div
                class="heatmap-cell"
                class:l1={day.intensity === 1}
                class:l2={day.intensity === 2}
                class:l3={day.intensity === 3}
                class:l4={day.intensity === 4}
                title="{day.date}: {day.intensity}"
              ></div>
            {/each}
          </div>
        </div>
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
      <div class="section-header">
        <h3>{i18n.t('habits-streak-rewards', 'Streak Rewards')}</h3>
        <button class="glass-btn" onclick={openAddReward}>{i18n.t('habits-new-reward', 'New Reward')}</button>
      </div>
      {#if rewards.length === 0}
        <p class="empty">{i18n.t('habits-no-rewards', 'No streak rewards configured.')}</p>
      {:else}
        {#each rewards as reward}
          <div class="reward-card">
            <div class="reward-header">
              <div>
                <span class="reward-habit">{reward.habit_name}</span>
                <span class="reward-type">{reward.is_consecutive ? i18n.t('habits-consecutive', 'Consecutive') : i18n.t('habits-accumulative', 'Accumulative')}</span>
              </div>
              <div class="reward-actions">
                <button class="icon-btn" onclick={() => openEditReward(reward)}>{i18n.t('habits-edit', 'Edit')}</button>
                <button class="icon-btn danger" onclick={() => deleteReward(reward.id)}>{i18n.t('habits-delete', 'Delete')}</button>
              </div>
            </div>
            <div class="reward-progress">
              <div class="reward-progress-text">
                <span>{i18n.t('habits-progress', 'Progress')}</span>
                <span class="reward-progress-count">{reward.current_progress} / {reward.target_days ?? reward.target_total ?? '?'} {i18n.t('habits-days-label', 'days')}</span>
              </div>
              <div class="progress-track">
                <div class="progress-fill" style="width: {rewardProgress(reward)}%"></div>
              </div>
            </div>
            {#each reward.milestones as ms}
              <div class="milestone" class:unlocked={ms.unlocked}>
                <span>{ms.target_days}d: {ms.reward_text}</span>
                {#if ms.unlocked}
                  <span class="unlocked-badge">{i18n.t('habits-unlocked', 'Unlocked')}</span>
                {/if}
              </div>
            {/each}
          </div>
        {/each}
      {/if}

      <div class="section-header" style="margin-top: 24px">
        <h3>{i18n.t('habits-goals', 'Goals')}</h3>
        <button class="glass-btn" onclick={openAddGoal}>{i18n.t('habits-new-goal', 'New Goal')}</button>
      </div>
      {#if goals.length === 0}
        <p class="empty">{i18n.t('habits-no-goals', 'No goals set.')}</p>
      {:else}
        {#each goals as goal}
          <div class="goal-card" class:completed={goal.is_completed}>
            <div class="goal-header">
              <div>
                <span class="goal-name">{goal.name}</span>
                {#if goal.deadline}
                  <span class="goal-deadline">{i18n.t('habits-due', 'Due:')} {goal.deadline}</span>
                {/if}
              </div>
              <div class="goal-actions">
                <button class="icon-btn" onclick={() => openEditGoal(goal)}>{i18n.t('habits-edit', 'Edit')}</button>
                {#if goal.is_completed}
                  <button class="icon-btn" onclick={() => archiveGoal(goal.id)}>{i18n.t('habits-archive', 'Archive')}</button>
                {/if}
                <button class="icon-btn danger" onclick={() => deleteGoal(goal.id)}>{i18n.t('habits-delete', 'Delete')}</button>
              </div>
            </div>
            {#if goal.description}
              <p class="goal-desc">{goal.description}</p>
            {/if}
            {#if goal.checkpoints.length > 0}
              {@const pct = goalProgress(goal)}
              <div class="goal-progress">
                <div class="reward-progress-text">
                  <span>{pct}%</span>
                  <span class="reward-progress-count">{goal.checkpoints.filter(c => c.completed).length} / {goal.checkpoints.length}</span>
                </div>
                <div class="progress-track">
                  <div class="progress-fill" class:complete={pct === 100} style="width: {pct}%"></div>
                </div>
              </div>
            {/if}
            <div class="checkpoints">
              {#each goal.checkpoints as cp}
                <label class="checkpoint">
                  <input type="checkbox" checked={cp.completed} onchange={() => toggleCheckpoint(goal.id, cp.id)} />
                  <span>{cp.description}</span>
                </label>
              {/each}
            </div>
            {#if !goal.is_completed}
              <button class="secondary-btn small" onclick={() => completeGoal(goal.id)}>{i18n.t('habits-mark-complete', 'Mark Complete')}</button>
            {/if}
          </div>
        {/each}
      {/if}
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
{#if showAddHabit}
  <div class="modal-backdrop" role="presentation" onclick={() => showAddHabit = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showAddHabit = false }}></div>
  <div class="modal-wrapper">
    <div class="modal">
      <h3>{editingHabit ? i18n.t('habits-edit-habit', 'Edit Habit') : i18n.t('habits-new-habit-modal', 'New Habit')}</h3>
    <div class="form-grid">
      <label>
        {i18n.t('habits-name', 'Name')}
        <input type="text" bind:value={habitName} placeholder={i18n.t('habits-habit-name-placeholder', 'Habit name')} />
      </label>
      <label>
        {i18n.t('habits-description', 'Description')}
        <input type="text" bind:value={habitDescription} placeholder={i18n.t('habits-desc-placeholder', 'Optional description')} />
      </label>
      <label>
        {i18n.t('habits-color', 'Color')}
        <div class="color-palette">
          {#each colors as c}
            <button
              class="color-swatch"
              class:selected={habitColor === c}
              style="background: {c}"
              aria-label="Color {c}"
              onclick={() => habitColor = c}
            ></button>
          {/each}
        </div>
      </label>
      <label>
        {i18n.t('habits-category', 'Category')}
        <input type="text" bind:value={habitCategory} placeholder={i18n.t('habits-category-placeholder', 'e.g. health, learning')} />
      </label>
    </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => showAddHabit = false}>{i18n.t('habits-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submitHabit} disabled={!habitName.trim()}>
          {editingHabit ? i18n.t('habits-update', 'Update') : i18n.t('habits-create', 'Create')}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Add Reward Modal -->
{#if showAddReward}
  <div class="modal-backdrop" role="presentation" onclick={() => showAddReward = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showAddReward = false }}></div>
  <div class="modal-wrapper">
    <div class="modal">
      <h3>{editingReward ? i18n.t('habits-edit-reward', 'Edit Streak Reward') : i18n.t('habits-new-reward-modal', 'New Streak Reward')}</h3>
      <div class="form-grid">
        <label>
          {i18n.t('habits-habit', 'Habit')}
          <select bind:value={rewardHabitId}>
            {#if habitsData?.habits}
              {#each habitsData.habits as habit}
                <option value={habit.id}>{habit.name}</option>
              {/each}
            {/if}
          </select>
        </label>
        <label>
          <input type="checkbox" bind:checked={rewardConsecutive} />
          <span>{i18n.t('habits-consecutive-days', 'Consecutive days (vs Accumulative)')}</span>
        </label>
        <label>
          {i18n.t('habits-target-days', 'Target Days')}
          <input type="number" bind:value={rewardTargetDays} placeholder={i18n.t('habits-target-days-placeholder', 'e.g., 7, 30, 100')} />
        </label>
        <label>
          {i18n.t('habits-target-total', 'Target Total (optional)')}
          <input type="number" bind:value={rewardTargetTotal} placeholder={i18n.t('habits-target-total-placeholder', 'Alternative count metric')} />
        </label>
      </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => showAddReward = false}>{i18n.t('habits-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submitReward} disabled={!rewardHabitId || !rewardTargetDays}>
          {editingReward ? i18n.t('habits-update', 'Update') : i18n.t('habits-create', 'Create')}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Add Goal Modal -->
{#if showAddGoal}
  <div class="modal-backdrop" role="presentation" onclick={() => showAddGoal = false} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showAddGoal = false }}></div>
  <div class="modal-wrapper">
    <div class="modal">
      <h3>{editingGoal ? i18n.t('habits-edit-goal', 'Edit Goal') : i18n.t('habits-new-goal-modal', 'New Goal')}</h3>
      <div class="form-grid">
        <label>
          {i18n.t('habits-goal-name', 'Goal Name')}
          <input type="text" bind:value={goalName} placeholder={i18n.t('habits-goal-name-placeholder', 'e.g., Complete certification')} />
        </label>
        <label>
          {i18n.t('habits-description', 'Description')}
          <input type="text" bind:value={goalDescription} placeholder={i18n.t('habits-goal-desc-placeholder', 'Optional details')} />
        </label>
        <label>
          {i18n.t('habits-reward-text', 'Reward Text')}
          <input type="text" bind:value={goalRewardText} placeholder={i18n.t('habits-reward-text-placeholder', "What you'll reward yourself with")} />
        </label>
        <label>
          {i18n.t('habits-deadline', 'Deadline (optional)')}
          <input type="date" bind:value={goalDeadline} />
        </label>
      </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => showAddGoal = false}>{i18n.t('habits-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submitGoal} disabled={!goalName.trim()}>
          {editingGoal ? i18n.t('habits-update', 'Update') : i18n.t('habits-create', 'Create')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .page { padding: 24px 32px; max-width: 1000px; width: 100%; margin: 0 auto; }

  .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  h2 { font-size: 1.3rem; letter-spacing: 0.2em; color: var(--text-primary); margin: 0; }

  .month-nav { display: flex; align-items: center; gap: 12px; }
  .nav-arrow { background: none; border: none; color: var(--text-secondary); cursor: pointer; padding: 4px; display: flex; transition: color 0.15s; }
  .nav-arrow:hover { color: var(--text-primary); }
  .nav-arrow svg { width: 18px; height: 18px; }
  .month-label { font-size: 0.9rem; color: var(--text-secondary); min-width: 140px; text-align: center; }

  .loading { text-align: center; padding: 48px; color: var(--text-tertiary); }
  .empty { text-align: center; padding: 48px; color: var(--text-tertiary); }

  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .section-header h3 { font-size: 0.85rem; color: var(--text-secondary); text-transform: uppercase; margin: 0; }

  /* Habit grid */
  .habit-grid {
    overflow-x: auto; margin-bottom: 24px;
    background: var(--glass); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    padding: 14px 16px; box-shadow: var(--glass-glow);
  }
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

  /* Heatmap */
  .chart-section { margin-bottom: 24px; }
  .chart-section h3 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin: 0; }
  .heatmap-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
  .heatmap-nav { display: flex; align-items: center; gap: 8px; }
  .heatmap-year { font-size: 0.85rem; color: var(--text-secondary); min-width: 40px; text-align: center; }
  .heatmap { display: flex; flex-wrap: wrap; gap: 2px; }
  .heatmap-cell { width: 12px; height: 12px; border-radius: 2px; background: var(--glass); }
  .heatmap-cell.l1 { background: rgba(14, 68, 41, 0.4); }
  .heatmap-cell.l2 { background: rgba(0, 109, 50, 0.6); }
  .heatmap-cell.l3 { background: rgba(38, 166, 65, 0.8); }
  .heatmap-cell.l4 { background: rgba(57, 211, 83, 1.0); box-shadow: 0 0 4px rgba(57, 211, 83, 0.3); }

  /* Summary card */
  .summary-card {
    background: var(--glass); backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    padding: 20px; margin-bottom: 20px; box-shadow: var(--glass-glow);
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
    background: var(--glass); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    padding: 16px; box-shadow: var(--glass-glow);
  }
  .chart-card h3 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin: 0 0 8px; }
  .analytics-card {
    background: var(--glass); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    padding: 16px; grid-column: 1 / -1; box-shadow: var(--glass-glow);
  }
  .insight { font-size: 0.85rem; color: var(--text-secondary); margin: 4px 0; }

  /* Rewards */
  .rewards-section h3 { font-size: 0.85rem; color: var(--text-tertiary); text-transform: uppercase; margin: 20px 0 12px; }
  .reward-card {
    background: var(--glass); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    padding: 16px; margin-bottom: 12px; box-shadow: var(--glass-glow);
  }
  .reward-header { display: flex; justify-content: space-between; margin-bottom: 8px; }
  .reward-actions, .goal-actions { display: flex; gap: 4px; }
  .reward-habit { font-weight: 600; color: var(--text-primary); }
  .reward-type { font-size: 0.75rem; color: var(--text-tertiary); }
  .reward-progress { margin-bottom: 10px; }
  .reward-progress-text {
    display: flex; justify-content: space-between; font-size: 0.8rem;
    color: var(--text-secondary); margin-bottom: 6px;
  }
  .reward-progress-count { color: var(--text-tertiary); font-variant-numeric: tabular-nums; }
  .progress-track {
    width: 100%; height: 6px; border-radius: 3px;
    background: var(--glass-border); overflow: hidden;
  }
  .progress-fill {
    height: 100%; background: linear-gradient(90deg, var(--accent), var(--accent-hover));
    border-radius: 3px; transition: width 0.4s cubic-bezier(0.16, 1, 0.3, 1);
    box-shadow: 0 0 8px var(--accent-glow);
  }
  .progress-fill.complete {
    background: linear-gradient(90deg, var(--success), #22c55e);
    box-shadow: 0 0 8px rgba(74, 222, 128, 0.3);
  }
  .goal-progress { margin-bottom: 12px; }
  .milestone { display: flex; justify-content: space-between; padding: 6px 0; font-size: 0.85rem; color: var(--text-secondary); border-bottom: 1px solid var(--glass-border); }
  .milestone.unlocked { color: var(--success); }
  .unlocked-badge { font-size: 0.7rem; color: var(--success); }

  .goal-card {
    background: var(--glass); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    padding: 16px; margin-bottom: 12px; box-shadow: var(--glass-glow);
  }
  .goal-card.completed { opacity: 0.6; }
  .goal-header { display: flex; justify-content: space-between; margin-bottom: 4px; }
  .goal-name { font-weight: 600; color: var(--text-primary); }
  .goal-deadline { font-size: 0.75rem; color: var(--text-secondary); }
  .goal-desc { font-size: 0.85rem; color: var(--text-secondary); margin: 4px 0 8px; }
  .checkpoints { display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px; }
  .checkpoint { display: flex; align-items: center; gap: 8px; color: var(--text-secondary); cursor: pointer; font-size: 0.85rem; transition: color 0.15s; }
  .checkpoint input[type="checkbox"] {
    width: 16px; height: 16px; cursor: pointer; accent-color: var(--success);
  }
  .checkpoint input[type="checkbox"]:checked ~ span {
    color: var(--success);
    text-decoration: line-through;
  }

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

  /* Modal */
  .modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); backdrop-filter: blur(4px); z-index: 100; }
  .modal-wrapper {
    position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
    z-index: 101; pointer-events: none;
  }
  .modal-wrapper .modal {
    pointer-events: auto;
  }
  .modal {
    position: relative;
    background: var(--modal-bg);
    border: 1px solid var(--modal-border); border-radius: var(--radius-lg);
    padding: 28px; width: 400px; z-index: 101;
    box-shadow: var(--modal-shadow);
  }
  .modal h3 { margin: 0 0 20px; color: var(--text-primary); position: relative; z-index: 10; }

  .form-grid { display: flex; flex-direction: column; gap: 14px; position: relative; z-index: 10; }
  .form-grid label { display: flex; flex-direction: column; gap: 4px; font-size: 0.8rem; color: var(--text-secondary); }
  .form-grid input[type="text"],
  .form-grid input[type="number"],
  .form-grid input[type="date"] {
    padding: 10px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.9rem;
    transition: border-color 0.2s, box-shadow 0.2s;
  }
  .form-grid input[type="text"]:focus,
  .form-grid input[type="number"]:focus,
  .form-grid input[type="date"]:focus {
    border-color: var(--accent); outline: none; box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .form-grid input[type="checkbox"] {
    width: 18px; height: 18px; cursor: pointer; margin: 0;
  }
  .form-grid label:has(input[type="checkbox"]) {
    flex-direction: row; align-items: center; gap: 8px;
  }
  .form-grid label:has(input[type="checkbox"]) span {
    font-size: 0.9rem; color: var(--text-primary);
  }

  .color-palette { display: flex; gap: 6px; padding: 4px 0; }
  .color-swatch {
    width: 28px; height: 28px; border-radius: 50%; border: 2px solid transparent;
    cursor: pointer; padding: 0; transition: transform 0.15s, box-shadow 0.15s;
  }
  .color-swatch:hover { transform: scale(1.15); }
  .color-swatch.selected { border-color: var(--text-primary); box-shadow: 0 0 10px var(--accent-glow); }

  .modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; position: relative; z-index: 10; }

  .primary-btn {
    padding: 8px 18px; border: 1px solid var(--accent-border); border-radius: var(--radius-sm);
    background: var(--accent-bg); backdrop-filter: blur(8px);
    color: var(--text-on-accent); cursor: pointer; font-size: 0.85rem; font-weight: 500;
    transition: all 0.2s;
  }
  .primary-btn:hover:not(:disabled) { background: var(--accent-border); box-shadow: 0 0 16px var(--accent-glow); }
  .primary-btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .secondary-btn {
    padding: 8px 18px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: var(--text-secondary); cursor: pointer; font-size: 0.85rem;
    transition: all 0.15s;
  }
  .secondary-btn:hover { border-color: var(--glass-border-hover); color: var(--text-primary); }
  .secondary-btn.small { padding: 6px 14px; font-size: 0.8rem; }
</style>
