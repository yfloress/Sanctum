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
  import type { StreakRewardDto, HabitDto } from '../../lib/types'

  interface Props {
    rewards: StreakRewardDto[]
    habits: HabitDto[]
    onrefresh: () => Promise<void>
  }

  let { rewards, habits, onrefresh }: Props = $props()

  let showAddReward = $state(false)
  let editingReward = $state<StreakRewardDto | null>(null)
  let rewardHabitId = $state('')
  let rewardConsecutive = $state(true)
  let rewardTargetDays = $state('')
  let rewardTargetTotal = $state('')

  function rewardProgress(r: StreakRewardDto): number {
    const target = r.target_days ?? r.target_total ?? 1
    if (target <= 0) return 0
    return Math.min(100, Math.round((r.current_progress / target) * 100))
  }

  function openAddReward() {
    editingReward = null
    rewardHabitId = habits[0]?.id ?? ''
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
      await onrefresh()
      app.showToast(isEditing ? i18n.t('habits-toast-reward-updated', 'Reward updated') : i18n.t('habits-toast-reward-created', 'Reward created'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  async function deleteReward(id: string) {
    try {
      await habitsApi.deleteStreakReward(id)
      await onrefresh()
      app.showToast(i18n.t('habits-toast-reward-deleted', 'Reward deleted'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }
</script>

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
            {#each habits as habit}
              <option value={habit.id}>{habit.name}</option>
            {/each}
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

<style>
  .reward-card {
    position: relative;
    background: var(--card-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--glass-border); border-radius: var(--radius-lg);
    padding: 16px; margin-bottom: 12px; box-shadow: var(--card-shadow);
    overflow: hidden;
  }
  .reward-card::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 1px;
    background: var(--card-accent-line);
    opacity: 0.5;
  }
  .reward-header { display: flex; justify-content: space-between; margin-bottom: 8px; }
  .reward-habit { font-weight: 600; color: var(--text-primary); font-size: 0.95rem; }
  .reward-type { font-size: 0.75rem; color: var(--text-tertiary); margin-left: 8px; }
  .reward-actions { display: flex; gap: 6px; }

  .reward-progress { margin-bottom: 10px; }

  .milestone { display: flex; justify-content: space-between; padding: 6px 0; font-size: 0.85rem; color: var(--text-secondary); border-bottom: 1px solid var(--glass-border); }
  .milestone.unlocked { color: var(--success); }
  .unlocked-badge { font-size: 0.7rem; color: var(--success); }
</style>
