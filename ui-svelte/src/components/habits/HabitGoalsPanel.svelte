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
  import { app } from '../../lib/stores/app.svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import * as habitsApi from '../../lib/api/habits'
  import type { GoalDto } from '../../lib/types'

  interface Props {
    goals: GoalDto[]
    onrefresh: () => Promise<void>
    ongoalsupdate: (goals: GoalDto[]) => void
  }

  let { goals, onrefresh, ongoalsupdate }: Props = $props()

  let showAddGoal = $state(false)
  let editingGoal = $state<GoalDto | null>(null)
  let goalName = $state('')
  let goalDescription = $state('')
  let goalRewardText = $state('')
  let goalDeadline = $state('')

  function goalProgress(g: GoalDto): number {
    if (g.checkpoints.length === 0) return g.is_completed ? 100 : 0
    const done = g.checkpoints.filter(c => c.completed).length
    return Math.round((done / g.checkpoints.length) * 100)
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
      await onrefresh()
      app.showToast(isEditing ? i18n.t('habits-toast-goal-updated', 'Goal updated') : i18n.t('habits-toast-goal-created', 'Goal created'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteGoal(id: string) {
    try {
      await habitsApi.deleteGoal(id)
      await onrefresh()
      app.showToast(i18n.t('habits-toast-goal-deleted', 'Goal deleted'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function toggleCheckpoint(goalId: string, checkpointId: string) {
    try {
      await habitsApi.toggleCheckpoint(goalId, checkpointId)
      const updated = await habitsApi.fetchGoals()
      ongoalsupdate(updated)
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function completeGoal(id: string) {
    try {
      await habitsApi.completeGoal(id)
      const updated = await habitsApi.fetchGoals()
      ongoalsupdate(updated)
      app.showToast(i18n.t('habits-toast-goal-completed', 'Goal completed!'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function archiveGoal(id: string) {
    try {
      await habitsApi.archiveGoal(id)
      await onrefresh()
      app.showToast(i18n.t('habits-toast-goal-archived', 'Goal archived'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }
</script>

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
  .goal-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 16px; margin-bottom: 12px; box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .goal-card::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.5;
  }
  .goal-card.completed { opacity: 0.6; }
  .goal-header { display: flex; justify-content: space-between; margin-bottom: 4px; }
  .goal-name { font-weight: 600; color: var(--text-primary); }
  .goal-deadline { font-size: 0.75rem; color: var(--text-secondary); }
  .goal-desc { font-size: 0.85rem; color: var(--text-secondary); margin: 4px 0 8px; }
  .goal-progress { margin-bottom: 12px; }

  .checkpoints { display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px; }
  .checkpoint { display: flex; align-items: center; gap: 8px; color: var(--text-secondary); cursor: pointer; font-size: 0.85rem; transition: color 0.15s; }
  .checkpoint input[type="checkbox"] {
    width: 16px; height: 16px; cursor: pointer; accent-color: var(--success);
  }
  .checkpoint input[type="checkbox"]:checked ~ span {
    color: var(--success);
    text-decoration: line-through;
  }
</style>
