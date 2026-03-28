<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
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
  let habitColor = $state('#4f9cf7')
  let habitCategory = $state('general')

  const colors = ['#4f9cf7', '#4ade80', '#f87171', '#fbbf24', '#a78bfa', '#f472b6', '#34d399', '#fb923c']

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
      habitsData = await habitsApi.fetchHabits(month, year)
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
    habitColor = '#4f9cf7'
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
      app.showToast(editingHabit ? 'Habit updated' : 'Habit created')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteHabit(id: string) {
    try {
      await habitsApi.deleteHabit(id)
      if (selectedHabit?.id === id) { selectedHabit = null; summary = null }
      await load()
      app.showToast('Habit deleted')
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
      app.showToast('Goal completed!')
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  const monthNames = ['January','February','March','April','May','June','July','August','September','October','November','December']

  $effect(() => { load() })
  $effect(() => { if (activeTab === 'rewards') loadRewards() })
  $effect(() => { if (activeTab === 'history') loadHistory() })
</script>

<div class="page">
  <div class="page-header">
    <h2>HABITS</h2>
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

  <div class="tabs">
    {#each [['habits', 'Habits'], ['rewards', 'Rewards'], ['history', 'History']] as [key, label]}
      <button class="tab-btn" class:active={activeTab === key} onclick={() => activeTab = key as Tab}>{label}</button>
    {/each}
  </div>

  {#if loading}
    <div class="loading">Loading...</div>

  <!-- HABITS TAB -->
  {:else if activeTab === 'habits'}
    <div class="section-header">
      <h3>Daily Tracking</h3>
      <button class="primary-btn" onclick={openAddHabit}>New Habit</button>
    </div>

    {#if habitsData && habitsData.habits.length > 0}
      <div class="habit-grid">
        <div class="grid-header">
          <span class="habit-name-col">Habit</span>
          {#each Array(habitsData.days_in_month) as _, i}
            <span class="day-num">{i + 1}</span>
          {/each}
        </div>
        {#each habitsData.habits as habit}
          <div class="grid-row">
            <button
              class="habit-name-col clickable"
              onclick={() => selectHabit(habit)}
              style="border-left: 3px solid {habit.color}"
            >
              {habit.name}
            </button>
            {#each habit.days.slice(1, habitsData.days_in_month + 1) as done, i}
              <button
                class="day-cell"
                class:done
                style={done ? `background: ${habit.color}` : ''}
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
          <h3>Activity Heatmap ({heatmap.year})</h3>
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
              <button class="icon-btn" onclick={() => openEditHabit(selectedHabit!)}>Edit</button>
              <button class="icon-btn danger" onclick={() => deleteHabit(selectedHabit!.id)}>Delete</button>
            </div>
          </div>
          <div class="stats-grid">
            <div class="stat"><span class="stat-val">{summary.current_streak}</span><span class="stat-lbl">Current Streak</span></div>
            <div class="stat"><span class="stat-val">{summary.best_streak}</span><span class="stat-lbl">Best Streak</span></div>
            <div class="stat"><span class="stat-val">{(summary.completion_rate * 100).toFixed(0)}%</span><span class="stat-lbl">Completion</span></div>
            <div class="stat"><span class="stat-val">{summary.last_30_days}</span><span class="stat-lbl">Last 30 Days</span></div>
          </div>
        </div>
      {/if}

      <!-- Analytics -->
      {#if analytics}
        <div class="analytics-section">
          {#if analytics.radar.categories.length > 0}
            <div class="chart-card">
              <h3>Habit Radar</h3>
              <RadarChart data={analytics.radar} />
            </div>
          {/if}
          {#if analytics.weekday_efficiency.labels.length > 0}
            <div class="chart-card">
              <h3>Weekday Efficiency</h3>
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
      <p class="empty">No habits yet. Create your first habit to start tracking.</p>
    {/if}

  <!-- REWARDS TAB -->
  {:else if activeTab === 'rewards'}
    <div class="rewards-section">
      <h3>Streak Rewards</h3>
      {#if rewards.length === 0}
        <p class="empty">No streak rewards configured.</p>
      {:else}
        {#each rewards as reward}
          <div class="reward-card">
            <div class="reward-header">
              <span class="reward-habit">{reward.habit_name}</span>
              <span class="reward-type">{reward.is_consecutive ? 'Consecutive' : 'Accumulative'}</span>
            </div>
            <div class="reward-progress">
              Progress: {reward.current_progress} / {reward.target_days ?? reward.target_total ?? '?'} days
            </div>
            {#each reward.milestones as ms}
              <div class="milestone" class:unlocked={ms.unlocked}>
                <span>{ms.target_days}d: {ms.reward_text}</span>
                {#if ms.unlocked}
                  <span class="unlocked-badge">Unlocked</span>
                {/if}
              </div>
            {/each}
          </div>
        {/each}
      {/if}

      <h3>Goals</h3>
      {#if goals.length === 0}
        <p class="empty">No goals set.</p>
      {:else}
        {#each goals as goal}
          <div class="goal-card" class:completed={goal.is_completed}>
            <div class="goal-header">
              <span class="goal-name">{goal.name}</span>
              {#if goal.deadline}
                <span class="goal-deadline">Due: {goal.deadline}</span>
              {/if}
            </div>
            {#if goal.description}
              <p class="goal-desc">{goal.description}</p>
            {/if}
            <div class="checkpoints">
              {#each goal.checkpoints as cp}
                <button class="checkpoint" class:done={cp.completed} onclick={() => toggleCheckpoint(goal.id, cp.id)}>
                  <span class="check-icon">{cp.completed ? '[x]' : '[ ]'}</span>
                  <span>{cp.description}</span>
                </button>
              {/each}
            </div>
            {#if !goal.is_completed}
              <button class="secondary-btn small" onclick={() => completeGoal(goal.id)}>Mark Complete</button>
            {/if}
          </div>
        {/each}
      {/if}
    </div>

  <!-- HISTORY TAB -->
  {:else if activeTab === 'history'}
    {#if achievements.length === 0}
      <p class="empty">No achievements yet.</p>
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
  <div class="modal">
    <h3>{editingHabit ? 'Edit Habit' : 'New Habit'}</h3>
    <div class="form-grid">
      <label>
        Name
        <input type="text" bind:value={habitName} placeholder="Habit name" />
      </label>
      <label>
        Description
        <input type="text" bind:value={habitDescription} placeholder="Optional description" />
      </label>
      <label>
        Color
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
        Category
        <input type="text" bind:value={habitCategory} placeholder="e.g. health, learning" />
      </label>
    </div>
    <div class="modal-actions">
      <button class="secondary-btn" onclick={() => showAddHabit = false}>Cancel</button>
      <button class="primary-btn" onclick={submitHabit} disabled={!habitName.trim()}>
        {editingHabit ? 'Update' : 'Create'}
      </button>
    </div>
  </div>
{/if}

<style>
  .page { padding: 24px 32px; max-width: 1000px; }

  .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  h2 { font-size: 1.3rem; letter-spacing: 0.2em; color: #e0e0e0; margin: 0; }

  .month-nav { display: flex; align-items: center; gap: 12px; }
  .nav-arrow { background: none; border: none; color: #888; cursor: pointer; padding: 4px; display: flex; }
  .nav-arrow:hover { color: #e0e0e0; }
  .nav-arrow svg { width: 18px; height: 18px; }
  .month-label { font-size: 0.9rem; color: #ccc; min-width: 140px; text-align: center; }

  .tabs {
    display: flex; gap: 4px; background: #111; border-radius: 8px;
    padding: 3px; border: 1px solid #222; margin-bottom: 24px; width: fit-content;
  }
  .tab-btn {
    padding: 8px 20px; border: none; border-radius: 6px; background: none;
    color: #888; cursor: pointer; font-size: 0.85rem; font-weight: 500;
  }
  .tab-btn.active { background: #1a1a1a; color: #e0e0e0; }

  .loading { text-align: center; padding: 48px; color: #666; }
  .empty { text-align: center; padding: 48px; color: #555; }

  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .section-header h3 { font-size: 0.85rem; color: #888; text-transform: uppercase; margin: 0; }

  /* Habit grid */
  .habit-grid { overflow-x: auto; margin-bottom: 24px; }
  .grid-header, .grid-row { display: flex; align-items: center; gap: 2px; min-width: max-content; }
  .grid-header { margin-bottom: 4px; }
  .habit-name-col { width: 120px; flex-shrink: 0; font-size: 0.8rem; color: #888; padding: 4px 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .habit-name-col.clickable { background: none; border: none; cursor: pointer; color: #ccc; text-align: left; border-radius: 4px; }
  .habit-name-col.clickable:hover { background: #1a1a1a; }
  .day-num { width: 24px; height: 20px; display: flex; align-items: center; justify-content: center; font-size: 0.65rem; color: #555; }
  .day-cell {
    width: 24px; height: 24px; border-radius: 4px; border: 1px solid #222;
    background: #111; cursor: pointer; padding: 0;
    transition: all 0.1s;
  }
  .day-cell:hover { border-color: #444; }
  .day-cell.done { border-color: transparent; }

  /* Heatmap */
  .chart-section { margin-bottom: 24px; }
  .chart-section h3 { font-size: 0.8rem; color: #666; text-transform: uppercase; margin-bottom: 8px; }
  .heatmap { display: flex; flex-wrap: wrap; gap: 2px; }
  .heatmap-cell { width: 12px; height: 12px; border-radius: 2px; background: #1a1a1a; }
  .heatmap-cell.l1 { background: #0e4429; }
  .heatmap-cell.l2 { background: #006d32; }
  .heatmap-cell.l3 { background: #26a641; }
  .heatmap-cell.l4 { background: #39d353; }

  /* Summary card */
  .summary-card { background: #111; border: 1px solid #222; border-radius: 10px; padding: 20px; margin-bottom: 20px; }
  .summary-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .summary-header h3 { margin: 0; font-size: 1rem; color: #e0e0e0; }
  .summary-actions { display: flex; gap: 8px; }
  .icon-btn { background: none; border: 1px solid #333; border-radius: 6px; color: #888; cursor: pointer; padding: 4px 12px; font-size: 0.8rem; }
  .icon-btn:hover { border-color: #555; color: #ccc; }
  .icon-btn.danger:hover { color: #f87171; border-color: #5a2d2d; }

  .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; }
  .stat { display: flex; flex-direction: column; align-items: center; }
  .stat-val { font-size: 1.4rem; font-weight: 700; color: #e0e0e0; }
  .stat-lbl { font-size: 0.7rem; color: #666; margin-top: 2px; }

  .analytics-section { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 24px; }
  .chart-card { background: #111; border: 1px solid #222; border-radius: 10px; padding: 16px; }
  .chart-card h3 { font-size: 0.8rem; color: #666; text-transform: uppercase; margin: 0 0 8px; }
  .analytics-card { background: #111; border: 1px solid #222; border-radius: 10px; padding: 16px; grid-column: 1 / -1; }
  .insight { font-size: 0.85rem; color: #aaa; margin: 4px 0; }

  /* Rewards */
  .rewards-section h3 { font-size: 0.85rem; color: #666; text-transform: uppercase; margin: 20px 0 12px; }
  .reward-card { background: #111; border: 1px solid #222; border-radius: 10px; padding: 16px; margin-bottom: 12px; }
  .reward-header { display: flex; justify-content: space-between; margin-bottom: 8px; }
  .reward-habit { font-weight: 600; color: #e0e0e0; }
  .reward-type { font-size: 0.75rem; color: #666; }
  .reward-progress { font-size: 0.85rem; color: #888; margin-bottom: 8px; }
  .milestone { display: flex; justify-content: space-between; padding: 6px 0; font-size: 0.85rem; color: #999; border-bottom: 1px solid #1a1a1a; }
  .milestone.unlocked { color: #4ade80; }
  .unlocked-badge { font-size: 0.7rem; color: #4ade80; }

  .goal-card { background: #111; border: 1px solid #222; border-radius: 10px; padding: 16px; margin-bottom: 12px; }
  .goal-card.completed { opacity: 0.6; }
  .goal-header { display: flex; justify-content: space-between; margin-bottom: 4px; }
  .goal-name { font-weight: 600; color: #e0e0e0; }
  .goal-deadline { font-size: 0.75rem; color: #888; }
  .goal-desc { font-size: 0.85rem; color: #888; margin: 4px 0 8px; }
  .checkpoints { display: flex; flex-direction: column; gap: 4px; margin-bottom: 12px; }
  .checkpoint { display: flex; align-items: center; gap: 8px; background: none; border: none; color: #999; cursor: pointer; padding: 4px 0; font-size: 0.85rem; text-align: left; }
  .checkpoint.done { color: #4ade80; }
  .check-icon { font-family: monospace; }

  /* Timeline */
  .timeline { display: flex; flex-direction: column; gap: 0; padding-left: 20px; }
  .timeline-item { display: flex; gap: 16px; padding: 12px 0; border-left: 2px solid #222; padding-left: 16px; position: relative; }
  .timeline-dot { position: absolute; left: -7px; top: 16px; width: 12px; height: 12px; border-radius: 50%; background: #4f9cf7; border: 2px solid #0a0a0a; }
  .timeline-content { display: flex; flex-direction: column; gap: 2px; }
  .ach-title { font-weight: 600; color: #e0e0e0; font-size: 0.9rem; }
  .ach-desc { font-size: 0.8rem; color: #888; }
  .ach-date { font-size: 0.7rem; color: #555; }

  /* Modal */
  .modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.6); z-index: 100; }
  .modal {
    position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
    background: #111; border: 1px solid #222; border-radius: 12px;
    padding: 28px; width: 400px; z-index: 101;
  }
  .modal h3 { margin: 0 0 20px; color: #e0e0e0; }

  .form-grid { display: flex; flex-direction: column; gap: 14px; }
  .form-grid label { display: flex; flex-direction: column; gap: 4px; font-size: 0.8rem; color: #888; }
  .form-grid input {
    padding: 10px 12px; border: 1px solid #333; border-radius: 6px;
    background: #0a0a0a; color: #e0e0e0; font-size: 0.9rem;
  }
  .form-grid input:focus { border-color: #4f9cf7; outline: none; }

  .color-palette { display: flex; gap: 6px; padding: 4px 0; }
  .color-swatch {
    width: 28px; height: 28px; border-radius: 50%; border: 2px solid transparent;
    cursor: pointer; padding: 0;
  }
  .color-swatch.selected { border-color: #fff; }

  .modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; }

  .primary-btn {
    padding: 8px 18px; border: none; border-radius: 6px;
    background: #4f9cf7; color: #fff; cursor: pointer; font-size: 0.85rem; font-weight: 500;
  }
  .primary-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .secondary-btn {
    padding: 8px 18px; border: 1px solid #333; border-radius: 6px;
    background: none; color: #ccc; cursor: pointer; font-size: 0.85rem;
  }
  .secondary-btn.small { padding: 6px 14px; font-size: 0.8rem; }
</style>
