<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
  import LiquidGlassButton from '../components/LiquidGlassButton.svelte'
  import LiquidGlassTab from '../components/LiquidGlassTab.svelte'
  import LiquidGlassBackground from '../components/LiquidGlassBackground.svelte'
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

  <LiquidGlassTab
    options={[
      { label: 'Habits', value: 'habits' },
      { label: 'Rewards', value: 'rewards' },
      { label: 'History', value: 'history' }
    ]}
    active={activeTab}
    onchange={(value) => activeTab = value as Tab}
  />

  {#if loading}
    <div class="loading">Loading...</div>

  <!-- HABITS TAB -->
  {:else if activeTab === 'habits'}
    <div class="section-header">
      <h3>Daily Tracking</h3>
      <LiquidGlassButton text="New Habit" contrast="dark" onclick={openAddHabit} />
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
  <div class="modal-wrapper">
    <div class="modal">
      <LiquidGlassBackground />
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
  </div>
{/if}

<style>
  .page { padding: 24px 32px; max-width: 1000px; }

  .page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  h2 { font-size: 1.3rem; letter-spacing: 0.2em; color: var(--text-primary); margin: 0; }

  .month-nav { display: flex; align-items: center; gap: 12px; }
  .nav-arrow { background: none; border: none; color: var(--text-secondary); cursor: pointer; padding: 4px; display: flex; transition: color 0.15s; }
  .nav-arrow:hover { color: var(--text-primary); }
  .nav-arrow svg { width: 18px; height: 18px; }
  .month-label { font-size: 0.9rem; color: #ccc; min-width: 140px; text-align: center; }

  .loading { text-align: center; padding: 48px; color: var(--text-tertiary); }
  .empty { text-align: center; padding: 48px; color: var(--text-tertiary); }

  .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .section-header h3 { font-size: 0.85rem; color: var(--text-secondary); text-transform: uppercase; margin: 0; }

  /* Habit grid */
  .habit-grid { overflow-x: auto; margin-bottom: 24px; }
  .grid-header, .grid-row { display: flex; align-items: center; gap: 2px; min-width: max-content; }
  .grid-header { margin-bottom: 4px; }
  .habit-name-col { width: 120px; flex-shrink: 0; font-size: 0.8rem; color: var(--text-secondary); padding: 4px 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .habit-name-col.clickable { background: none; border: none; cursor: pointer; color: #ccc; text-align: left; border-radius: 4px; transition: background 0.15s; }
  .habit-name-col.clickable:hover { background: var(--glass-hover); }
  .day-num { width: 24px; height: 20px; display: flex; align-items: center; justify-content: center; font-size: 0.65rem; color: var(--text-tertiary); }
  .day-cell {
    width: 24px; height: 24px; border-radius: 4px; border: 1px solid var(--glass-border);
    background: var(--glass); cursor: pointer; padding: 0;
    transition: all 0.15s;
  }
  .day-cell:hover { border-color: var(--glass-border-hover); background: var(--glass-hover); }
  .day-cell.done { border-color: transparent; box-shadow: 0 0 6px rgba(255,255,255,0.1); }

  /* Heatmap */
  .chart-section { margin-bottom: 24px; }
  .chart-section h3 { font-size: 0.8rem; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 8px; }
  .heatmap { display: flex; flex-wrap: wrap; gap: 2px; }
  .heatmap-cell { width: 12px; height: 12px; border-radius: 2px; background: var(--glass); }
  .heatmap-cell.l1 { background: rgba(14, 68, 41, 0.7); }
  .heatmap-cell.l2 { background: rgba(0, 109, 50, 0.7); }
  .heatmap-cell.l3 { background: rgba(38, 166, 65, 0.7); }
  .heatmap-cell.l4 { background: rgba(57, 211, 83, 0.8); box-shadow: 0 0 4px rgba(57, 211, 83, 0.3); }

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
  .icon-btn:hover { border-color: var(--glass-border-hover); color: #ccc; }
  .icon-btn.danger:hover { color: var(--danger); border-color: rgba(248, 113, 113, 0.3); }

  .stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; }
  .stat { display: flex; flex-direction: column; align-items: center; }
  .stat-val { font-size: 1.4rem; font-weight: 700; color: var(--text-primary); }
  .stat-lbl { font-size: 0.7rem; color: var(--text-tertiary); margin-top: 2px; }

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
  .insight { font-size: 0.85rem; color: #aaa; margin: 4px 0; }

  /* Rewards */
  .rewards-section h3 { font-size: 0.85rem; color: var(--text-tertiary); text-transform: uppercase; margin: 20px 0 12px; }
  .reward-card {
    background: var(--glass); backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    padding: 16px; margin-bottom: 12px; box-shadow: var(--glass-glow);
  }
  .reward-header { display: flex; justify-content: space-between; margin-bottom: 8px; }
  .reward-habit { font-weight: 600; color: var(--text-primary); }
  .reward-type { font-size: 0.75rem; color: var(--text-tertiary); }
  .reward-progress { font-size: 0.85rem; color: var(--text-secondary); margin-bottom: 8px; }
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
  .checkpoints { display: flex; flex-direction: column; gap: 4px; margin-bottom: 12px; }
  .checkpoint { display: flex; align-items: center; gap: 8px; background: none; border: none; color: var(--text-secondary); cursor: pointer; padding: 4px 0; font-size: 0.85rem; text-align: left; transition: color 0.15s; }
  .checkpoint.done { color: var(--success); }
  .check-icon { font-family: monospace; }

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
    background: linear-gradient(-75deg, rgba(0, 0, 0, 0.05), rgba(0, 0, 0, 0.2), rgba(0, 0, 0, 0.05));
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 28px; width: 400px; z-index: 101;
    box-shadow: inset 0 0.125em 0.125em rgba(254, 254, 254, 0.05), inset 0 -0.125em 0.125em rgba(0, 0, 0, 0.5), 0 0.25em 0.125em -0.125em rgba(254, 254, 254, 0.2), 0 0 0.1em 0.25em inset rgba(0, 0, 0, 0.2);
  }
  .modal h3 { margin: 0 0 20px; color: var(--text-primary); position: relative; z-index: 10; }

  .form-grid { display: flex; flex-direction: column; gap: 14px; position: relative; z-index: 10; }
  .form-grid label { display: flex; flex-direction: column; gap: 4px; font-size: 0.8rem; color: var(--text-secondary); }
  .form-grid input {
    padding: 10px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: rgba(0, 0, 0, 0.25); color: var(--text-primary); font-size: 0.9rem;
    transition: border-color 0.2s, box-shadow 0.2s;
  }
  .form-grid input:focus { border-color: var(--accent); outline: none; box-shadow: 0 0 0 3px var(--accent-glow); }

  .color-palette { display: flex; gap: 6px; padding: 4px 0; }
  .color-swatch {
    width: 28px; height: 28px; border-radius: 50%; border: 2px solid transparent;
    cursor: pointer; padding: 0; transition: transform 0.15s, box-shadow 0.15s;
  }
  .color-swatch:hover { transform: scale(1.15); }
  .color-swatch.selected { border-color: #fff; box-shadow: 0 0 10px rgba(255,255,255,0.2); }

  .modal-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; position: relative; z-index: 10; }

  .primary-btn {
    padding: 8px 18px; border: 1px solid rgba(79, 156, 247, 0.3); border-radius: var(--radius-sm);
    background: rgba(79, 156, 247, 0.2); backdrop-filter: blur(8px);
    color: #fff; cursor: pointer; font-size: 0.85rem; font-weight: 500;
    transition: all 0.2s;
  }
  .primary-btn:hover:not(:disabled) { background: rgba(79, 156, 247, 0.3); box-shadow: 0 0 16px var(--accent-glow); }
  .primary-btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .secondary-btn {
    padding: 8px 18px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: none; color: #ccc; cursor: pointer; font-size: 0.85rem;
    transition: all 0.15s;
  }
  .secondary-btn:hover { border-color: var(--glass-border-hover); }
  .secondary-btn.small { padding: 6px 14px; font-size: 0.8rem; }
</style>
