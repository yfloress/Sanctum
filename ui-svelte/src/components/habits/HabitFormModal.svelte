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
  import type { HabitDto } from '../../lib/types'

  const colors = ['#a855f7', '#4ade80', '#f87171', '#fbbf24', '#a78bfa', '#f472b6', '#34d399', '#fb923c']

  interface Props {
    show: boolean
    editing: HabitDto | null
    onsubmit: () => Promise<void>
    onclose: () => void
  }

  let { show = $bindable(false), editing, onsubmit, onclose }: Props = $props()

  let habitName = $state('')
  let habitDescription = $state('')
  let habitColor = $state('#a855f7')
  let habitCategory = $state('general')

  $effect(() => {
    show
    editing
    if (show) {
      if (editing) {
        habitName = editing.name
        habitDescription = editing.description ?? ''
        habitColor = editing.color
        habitCategory = editing.category
      } else {
        habitName = ''
        habitDescription = ''
        habitColor = '#a855f7'
        habitCategory = 'general'
      }
    }
  })

  async function submitHabit() {
    try {
      const desc = habitDescription || null
      if (editing) {
        await habitsApi.updateHabit(editing.id, habitName, desc, habitColor, habitCategory)
      } else {
        await habitsApi.createHabit(habitName, desc, habitColor, habitCategory)
      }
      show = false
      await onsubmit()
      app.showToast(editing ? i18n.t('habits-toast-habit-updated', 'Habit updated') : i18n.t('habits-toast-habit-created', 'Habit created'))
    } catch (e) {
      app.showToast(String(e), true)
    }
  }

  function close() {
    show = false
    onclose()
  }
</script>

{#if show}
  <div class="modal-backdrop" role="presentation" onclick={close} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') close() }}></div>
  <div class="modal-wrapper">
    <div class="modal">
      <h3>{editing ? i18n.t('habits-edit-habit', 'Edit Habit') : i18n.t('habits-new-habit-modal', 'New Habit')}</h3>
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
        <button class="secondary-btn" onclick={close}>{i18n.t('habits-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={submitHabit} disabled={!habitName.trim()}>
          {editing ? i18n.t('habits-update', 'Update') : i18n.t('habits-create', 'Create')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .color-palette { display: flex; gap: 6px; margin-top: 4px; }
  .color-swatch {
    width: 28px; height: 28px; border-radius: 50%; border: 2px solid transparent;
    cursor: pointer; transition: border-color 0.15s, transform 0.15s;
  }
  .color-swatch:hover { transform: scale(1.2); }
  .color-swatch.selected { border-color: var(--text-primary); transform: scale(1.15); }
</style>
