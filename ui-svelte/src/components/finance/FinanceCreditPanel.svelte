<!-- Sanctum — a privacy-first personal finance and crypto vault.
     Copyright (C) 2026  yfloress

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
  import { errorMessage } from '../../lib/errors'
  import { app } from '../../lib/stores/app.svelte'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import { dialog } from '../../lib/actions/dialog'
  import { mask } from '../../lib/currency'
  import * as financeApi from '../../lib/api/finance'
  import type {
    AccountDto, CategoriesResponse, CreditDto, CreditInstallmentDto, CreditKind
  } from '../../lib/types'
  import ConfirmDialog from '../ConfirmDialog.svelte'

  interface Props {
    accounts: AccountDto[]
    categories: CategoriesResponse | null
    /** Called after any change, so the ledger and balances refresh. */
    onchange: () => Promise<void>
  }

  let { accounts, categories, onchange }: Props = $props()

  let credits = $state<CreditDto[]>([])
  let showForm = $state(false)
  let saving = $state(false)
  let busyId = $state('')
  let pendingDelete = $state<CreditDto | null>(null)
  /** Credits whose schedule is unfolded. */
  let expanded = $state<string[]>([])
  /** Credits showing the interest-versus-principal breakdown. */
  let expandedTable = $state<string[]>([])

  // Form
  let kind = $state<CreditKind>('installments')
  let name = $state('')
  let accountId = $state('')
  let category = $state('')
  let downPayment = $state('')
  let downPaymentDate = $state(today())
  let installmentAmount = $state('')
  let installmentCount = $state('12')
  let firstDueDate = $state(today())
  let cashPrice = $state('')
  let principal = $state('')
  let rate = $state('')
  let ratePeriod = $state<'monthly' | 'annual'>('monthly')
  /** Set once the user overrides the suggested payment, which then stands. */
  let installmentTouched = $state(false)

  // Row being corrected, and the charge form's target credit.
  let editing = $state<CreditInstallmentDto | null>(null)
  let editAmount = $state('')
  let editDate = $state('')
  let charging = $state<CreditDto | null>(null)
  let chargeAmount = $state('')
  let chargeDate = $state(today())
  let chargeNote = $state('')

  let categoryOptions = $derived(categories?.expense ?? [])

  function today(): string {
    return new Date().toISOString().slice(0, 10)
  }

  /** Reads a typed figure, accepting both a comma and a dot decimal. */
  function parseNumber(raw: string): number | null {
    const cleaned = raw.trim().replace(/\s/g, '').replace(/\.(?=\d{3}\b)/g, '').replace(',', '.')
    if (!cleaned) return null
    const value = Number(cleaned)
    return Number.isFinite(value) ? value : null
  }

  function format(value: number): string {
    return value.toLocaleString(undefined, { maximumFractionDigits: 2 })
  }

  let count = $derived.by(() => {
    const parsed = parseInt(installmentCount, 10)
    return Number.isFinite(parsed) && parsed > 0 ? parsed : 0
  })

  /**
   * The payment a loan of this size at this rate would carry.
   *
   * The constant-payment method, the same one the backend uses. Only ever a
   * suggestion: rounding and fee conventions differ between lenders and between
   * markets, so the figure printed on the contract wins and can be typed over.
   */
  let suggestedInstallment = $derived.by(() => {
    if (kind !== 'loan') return null
    const amount = parseNumber(principal)
    const monthly = monthlyRate()
    if (amount === null || amount <= 0 || count < 1) return null
    if (monthly === 0) return amount / count
    return (amount * monthly) / (1 - Math.pow(1 + monthly, -count))
  })

  /**
   * The rate as a monthly fraction, whichever period it was quoted in.
   *
   * Quantised exactly as the backend does — percent to hundredths, then to
   * millionths, then an annual figure divided by twelve — so the payment
   * suggested here and the breakdown computed there cannot disagree.
   */
  function monthlyRate(): number {
    const value = parseNumber(rate)
    if (value === null || value <= 0) return 0
    const annualPpm = Math.round(value * 100) * 100
    const monthlyPpm = ratePeriod === 'annual' ? Math.floor((annualPpm + 6) / 12) : annualPpm
    return monthlyPpm / 1_000_000
  }

  // Fills the payment box from the suggestion until the user disagrees with it.
  $effect(() => {
    const suggestion = suggestedInstallment
    if (kind === 'loan' && !installmentTouched && suggestion !== null) {
      installmentAmount = suggestion.toFixed(2)
    }
  })

  /** What the whole plan adds up to, down payment included. */
  let previewTotal = $derived.by(() => {
    const payment = parseNumber(installmentAmount)
    if (payment === null || count < 1) return null
    return payment * count + (parseNumber(downPayment) ?? 0)
  })

  /** What the credit costs beyond the thing it buys. */
  let previewExtra = $derived.by(() => {
    const payment = parseNumber(installmentAmount)
    if (payment === null || count < 1) return null
    if (kind === 'loan') {
      const lent = parseNumber(principal)
      return lent === null ? null : payment * count - lent
    }
    const cash = parseNumber(cashPrice)
    return cash === null || previewTotal === null ? null : previewTotal - cash
  })

  async function load() {
    try {
      credits = await financeApi.fetchCredits()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  $effect(() => {
    load()
  })

  function resetForm() {
    showForm = false
    kind = 'installments'
    name = ''
    accountId = accounts[0]?.id ?? ''
    category = ''
    downPayment = ''
    downPaymentDate = today()
    installmentAmount = ''
    installmentCount = '12'
    firstDueDate = today()
    cashPrice = ''
    principal = ''
    rate = ''
    ratePeriod = 'monthly'
    installmentTouched = false
  }

  async function submit() {
    saving = true
    try {
      await financeApi.addCredit({
        account_id: accountId,
        name,
        category,
        kind,
        down_payment: downPayment || undefined,
        down_payment_date: downPayment ? downPaymentDate : undefined,
        installment_amount: installmentAmount,
        installment_count: count,
        first_due_date: firstDueDate,
        cash_price: kind === 'installments' ? cashPrice || undefined : undefined,
        principal: kind === 'loan' ? principal || undefined : undefined,
        rate: kind === 'loan' ? rate || undefined : undefined,
        rate_period: kind === 'loan' ? ratePeriod : undefined,
      })
      resetForm()
      await load()
      app.showToast(i18n.t('finances-credit-added', 'Credit saved'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      saving = false
    }
  }

  /** Pays the earliest row still owed, whatever kind it is. */
  async function payNext(credit: CreditDto) {
    const next = credit.installments.find(row => !row.is_paid)
    if (next) await pay(credit, next.id)
  }

  async function pay(credit: CreditDto, installmentId: string) {
    busyId = credit.id
    try {
      await financeApi.payInstallment(installmentId)
      await load()
      await onchange()
      app.showToast(i18n.t('finances-credit-paid', 'Installment paid'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      busyId = ''
    }
  }

  async function undo(credit: CreditDto, installmentId: string) {
    busyId = credit.id
    try {
      await financeApi.unpayInstallment(installmentId)
      await load()
      await onchange()
      app.showToast(i18n.t('finances-credit-undone', 'Payment undone'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    } finally {
      busyId = ''
    }
  }

  function startEdit(row: CreditInstallmentDto) {
    editing = row
    editAmount = row.amount_raw
    editDate = row.due_date
  }

  async function saveEdit() {
    if (!editing) return
    try {
      await financeApi.updateInstallment({
        installment_id: editing.id,
        amount: editAmount,
        due_date: editDate,
      })
      editing = null
      await load()
      app.showToast(i18n.t('finances-credit-updated', 'Installment updated'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  function startCharge(credit: CreditDto) {
    charging = credit
    chargeAmount = ''
    chargeDate = today()
    chargeNote = ''
  }

  async function saveCharge() {
    if (!charging) return
    try {
      await financeApi.addCreditCharge({
        credit_id: charging.id,
        amount: chargeAmount,
        date: chargeDate,
        note: chargeNote,
      })
      charging = null
      await load()
      app.showToast(i18n.t('finances-credit-charge-added', 'Charge recorded'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function removeCharge(installmentId: string) {
    try {
      await financeApi.deleteCreditCharge(installmentId)
      await load()
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  async function remove(credit: CreditDto) {
    try {
      await financeApi.deleteCredit(credit.id)
      await load()
      app.showToast(i18n.t('finances-credit-deleted', 'Credit deleted'))
    } catch (e) {
      app.showToast(errorMessage(e), true)
    }
  }

  function toggle(list: string[], id: string): string[] {
    return list.includes(id) ? list.filter(v => v !== id) : [...list, id]
  }

  function statusLabel(credit: CreditDto): string {
    switch (credit.status) {
      case 'done':
        return i18n.t('finances-credit-done', 'Finished')
      case 'overdue':
        return i18n.tArgs(
          'finances-credit-overdue',
          { count: credit.overdue_count },
          `Overdue: ${credit.overdue_count}`
        )
      case 'ahead':
        return i18n.t('finances-credit-ahead', 'Ahead of schedule')
      default:
        return i18n.t('finances-credit-on-track', 'Up to date')
    }
  }

  function rowLabel(row: CreditInstallmentDto, credit: CreditDto): string {
    if (row.kind === 'down_payment') return i18n.t('finances-credit-down-payment', 'Down payment')
    if (row.kind === 'charge') {
      return row.note || i18n.t('finances-credit-charge', 'Charge')
    }
    return `${row.number}/${credit.installment_count}`
  }

  let formValid = $derived(
    !!accountId && !!name.trim() && !!category && !!installmentAmount && count > 0 &&
    (kind === 'installments' || !!principal)
  )
</script>

<div class="settings-card">
  <div class="credit-head">
    <div>
      <h3 class="settings-card-title">{i18n.t('finances-credits', 'Credits')}</h3>
      <p class="credit-note">
        {i18n.t('finances-credits-desc', 'Debts you pay off over a fixed number of dated payments. Marking one as paid writes the expense into the account it comes out of.')}
      </p>
    </div>
    {#if !showForm}
      <button class="glass-btn" onclick={() => { resetForm(); showForm = true }} disabled={accounts.length === 0}>
        {i18n.t('finances-credit-new', 'New')}
      </button>
    {/if}
  </div>

  {#if accounts.length === 0}
    <p class="empty">{i18n.t('finances-no-accounts-create', 'No accounts yet. Create your first account.')}</p>
  {/if}

  {#if showForm}
    <div class="credit-form">
      <!-- Which of the two things the lender told you. Everything below follows
           from this, because no lender ever states both. -->
      <div class="toggle-row">
        <button class="toggle-btn" class:active={kind === 'installments'}
          onclick={() => { kind = 'installments'; installmentTouched = false }}>
          {i18n.t('finances-credit-kind-installments', 'They told me the payment')}
        </button>
        <button class="toggle-btn" class:active={kind === 'loan'}
          onclick={() => { kind = 'loan'; installmentTouched = false }}>
          {i18n.t('finances-credit-kind-loan', 'They told me the rate')}
        </button>
      </div>

      <input type="text" placeholder={i18n.t('finances-credit-name', 'What did you buy?')} bind:value={name} />
      <select bind:value={accountId} aria-label={i18n.t('finances-account', 'Account')}>
        {#each accounts as acc}
          <option value={acc.id}>{acc.name}</option>
        {/each}
      </select>
      <select bind:value={category} aria-label={i18n.t('finances-category', 'Category')}>
        <option value="">{i18n.t('finances-select', 'Select...')}</option>
        {#each categoryOptions as cat}
          <option value={cat.name}>{cat.label}</option>
        {/each}
      </select>

      <div class="credit-pair">
        <label>
          <span>{i18n.t('finances-credit-down-payment-optional', 'Paid up front (optional)')}</span>
          <input type="text" inputmode="decimal" placeholder="0" bind:value={downPayment} />
        </label>
        {#if downPayment}
          <label>
            <span>{i18n.t('finances-credit-down-payment-date', 'Handed over on')}</span>
            <input type="date" bind:value={downPaymentDate} />
          </label>
        {/if}
      </div>

      {#if kind === 'loan'}
        <label>
          <span>{i18n.t('finances-credit-principal', 'Amount financed')}</span>
          <input type="text" inputmode="decimal" placeholder="0.00" bind:value={principal} />
        </label>
        <div class="credit-pair">
          <label>
            <span>{i18n.t('finances-credit-rate', 'Interest rate (%)')}</span>
            <input type="text" inputmode="decimal" placeholder="0.00" bind:value={rate} />
          </label>
          <label>
            <span>{i18n.t('finances-credit-rate-period', 'Quoted as')}</span>
            <select bind:value={ratePeriod}>
              <option value="monthly">{i18n.t('finances-credit-rate-monthly', 'Monthly')}</option>
              <option value="annual">{i18n.t('finances-credit-rate-annual', 'Annual')}</option>
            </select>
          </label>
        </div>
      {/if}

      <div class="credit-pair">
        <label>
          <span>{i18n.t('finances-credit-installment', 'Amount per installment')}</span>
          <input type="text" inputmode="decimal" placeholder="0.00" bind:value={installmentAmount}
            oninput={() => (installmentTouched = true)} />
        </label>
        <label>
          <span>{i18n.t('finances-credit-count', 'How many')}</span>
          <input type="number" min="1" max="120" bind:value={installmentCount} />
        </label>
      </div>

      {#if kind === 'loan' && suggestedInstallment !== null}
        <p class="credit-hint">
          {i18n.t('finances-credit-suggested', 'Calculated from the rate. If your contract says something else, type that instead.')}
        </p>
      {/if}

      <label class="credit-date">
        <span>{i18n.t('finances-credit-first-due', 'First one due on')}</span>
        <input type="date" bind:value={firstDueDate} />
      </label>

      {#if kind === 'installments'}
        <label>
          <span>{i18n.t('finances-credit-cash-price', 'Price if you paid it all at once (optional)')}</span>
          <input type="text" inputmode="decimal" placeholder="0.00" bind:value={cashPrice} />
        </label>
      {/if}

      {#if previewTotal !== null}
        <div class="credit-preview">
          <span>{i18n.t('finances-credit-total', 'Total')}: {format(previewTotal)}</span>
          {#if previewExtra !== null && previewExtra > 0}
            <!-- The figure nobody is shown at the counter, which is the whole
                 reason for asking for a cash price or a rate at all. -->
            <span class="credit-extra">
              {i18n.tArgs(
                'finances-credit-extra',
                { amount: format(previewExtra) },
                `You pay ${format(previewExtra)} more than the thing costs`
              )}
            </span>
          {/if}
        </div>
      {/if}

      <div class="credit-actions">
        <button class="secondary-btn" onclick={resetForm} disabled={saving}>
          {i18n.t('finances-cancel', 'Cancel')}
        </button>
        <button class="primary-btn" onclick={submit} disabled={saving || !formValid}>
          {i18n.t('finances-create', 'Create')}
        </button>
      </div>
    </div>
  {/if}

  {#if credits.length === 0 && !showForm && accounts.length > 0}
    <p class="empty">{i18n.t('finances-no-credits', 'No credits yet.')}</p>
  {/if}

  <div class="credit-list">
    {#each credits as credit (credit.id)}
      <div class="credit-card" class:done={credit.status === 'done'}>
        <div class="credit-title">
          <div class="credit-name-wrap">
            <span class="credit-name">{credit.name}</span>
            <span class="credit-account">{credit.account_name} · {credit.category_label}</span>
          </div>
          <button class="delete-btn" onclick={() => (pendingDelete = credit)} aria-label={i18n.t('confirm-delete-button', 'Delete')}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
          </button>
        </div>

        <div class="credit-bar">
          <div class="credit-fill" class:overdue={credit.status === 'overdue'}
            style="width: {credit.percentage}%"></div>
        </div>

        <div class="credit-figures">
          <span>{credit.paid_count}/{credit.installment_count} {i18n.t('finances-credit-installments', 'installments')}</span>
          <span>{mask(credit.paid)} · {mask(credit.remaining)} {i18n.t('finances-credit-left', 'left')}</span>
        </div>

        <div class="credit-status-row">
          <span class="credit-status" class:overdue={credit.status === 'overdue'}
            class:ahead={credit.status === 'ahead'} class:done={credit.status === 'done'}>
            {statusLabel(credit)}
          </span>
          {#if credit.next_due_date}
            <span class="credit-next">{i18n.t('finances-credit-next', 'Next')}: {credit.next_due_date}</span>
          {/if}
          {#if credit.monthly_rate}
            <span class="credit-next">{credit.monthly_rate}% {i18n.t('finances-credit-rate-monthly-short', 'monthly')}</span>
          {/if}
          {#if credit.interest}
            <span class="credit-next">{i18n.t('finances-credit-interest', 'Interest')}: {mask(credit.interest)}</span>
          {/if}
          {#if credit.charges}
            <span class="credit-status overdue">{i18n.t('finances-credit-charges', 'Charges')}: {mask(credit.charges)}</span>
          {/if}
        </div>

        <div class="credit-buttons">
          <button class="glass-btn" onclick={() => (expanded = toggle(expanded, credit.id))}>
            {expanded.includes(credit.id)
              ? i18n.t('finances-credit-hide-schedule', 'Hide schedule')
              : i18n.t('finances-credit-show-schedule', 'Schedule')}
          </button>
          {#if credit.amortization.length > 0}
            <button class="glass-btn" onclick={() => (expandedTable = toggle(expandedTable, credit.id))}>
              {i18n.t('finances-credit-breakdown', 'Breakdown')}
            </button>
          {/if}
          <button class="glass-btn" onclick={() => startCharge(credit)}>
            {i18n.t('finances-credit-add-charge', 'Add charge')}
          </button>
          {#if credit.status !== 'done'}
            <button class="primary-btn" onclick={() => payNext(credit)} disabled={busyId === credit.id}>
              {i18n.t('finances-credit-pay-next', 'Pay next')}
            </button>
          {/if}
        </div>

        {#if expanded.includes(credit.id)}
          <div class="credit-schedule">
            {#each credit.installments as row (row.id)}
              <div class="credit-inst" class:paid={row.is_paid} class:overdue={row.is_overdue}
                class:charge={row.kind === 'charge'}>
                <span class="credit-inst-num">{rowLabel(row, credit)}</span>
                <span class="credit-inst-date">{row.paid_date ?? row.due_date}</span>
                <span class="credit-inst-amount">{mask(row.amount)}</span>
                <span class="credit-inst-state">
                  {#if row.is_paid}
                    {i18n.t('finances-credit-inst-paid', 'Paid')}
                  {:else if row.is_overdue}
                    {i18n.t('finances-credit-inst-overdue', 'Overdue')}
                  {:else}
                    {i18n.t('finances-credit-inst-pending', 'Pending')}
                  {/if}
                </span>
                {#if row.is_paid}
                  <button class="credit-inst-btn" onclick={() => undo(credit, row.id)} disabled={busyId === credit.id}>
                    {i18n.t('finances-credit-undo', 'Undo')}
                  </button>
                {:else}
                  <button class="credit-inst-btn" onclick={() => startEdit(row)}>
                    {i18n.t('finances-credit-edit', 'Edit')}
                  </button>
                  <button class="credit-inst-btn" onclick={() => pay(credit, row.id)} disabled={busyId === credit.id}>
                    {i18n.t('finances-credit-pay-short', 'Pay')}
                  </button>
                  {#if row.kind === 'charge'}
                    <button class="credit-inst-btn danger" onclick={() => removeCharge(row.id)}>
                      {i18n.t('action-delete', 'Delete')}
                    </button>
                  {/if}
                {/if}
              </div>
            {/each}
          </div>
        {/if}

        {#if expandedTable.includes(credit.id)}
          <div class="credit-table-wrap">
            <table class="credit-table">
              <thead>
                <tr>
                  <th>#</th>
                  <th>{i18n.t('finances-credit-col-payment', 'Payment')}</th>
                  <th>{i18n.t('finances-credit-col-interest', 'Interest')}</th>
                  <th>{i18n.t('finances-credit-col-principal', 'Principal')}</th>
                  <th>{i18n.t('finances-credit-col-balance', 'Still owed')}</th>
                </tr>
              </thead>
              <tbody>
                {#each credit.amortization as line (line.number)}
                  <tr>
                    <td>{line.number}</td>
                    <td>{mask(line.payment)}</td>
                    <td>{mask(line.interest)}</td>
                    <td>{mask(line.principal)}</td>
                    <td>{mask(line.balance)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

{#if editing}
  <div class="modal-backdrop" role="presentation" onclick={() => (editing = null)}></div>
  <div class="modal-wrapper">
    <div class="modal" use:dialog={{ onclose: () => (editing = null) }}>
      <h3>{i18n.t('finances-credit-edit-title', 'Correct this installment')}</h3>
      <p class="credit-note">
        {i18n.t('finances-credit-edit-desc', 'For a plan whose payments are not all equal: a larger final one, a skipped month, a figure that came out different.')}
      </p>
      <div class="credit-dialog-fields">
        <label>
          <span>{i18n.t('finances-credit-installment', 'Amount per installment')}</span>
          <input type="text" inputmode="decimal" bind:value={editAmount} />
        </label>
        <label>
          <span>{i18n.t('finances-credit-due-date', 'Due on')}</span>
          <input type="date" bind:value={editDate} />
        </label>
      </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => (editing = null)}>{i18n.t('finances-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={saveEdit} disabled={!editAmount || !editDate}>
          {i18n.t('finances-save', 'Save')}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if charging}
  <div class="modal-backdrop" role="presentation" onclick={() => (charging = null)}></div>
  <div class="modal-wrapper">
    <div class="modal" use:dialog={{ onclose: () => (charging = null) }}>
      <h3>{i18n.t('finances-credit-charge-title', 'Record a charge')}</h3>
      <p class="credit-note">
        {i18n.t('finances-credit-charge-desc', 'A late fee or interest the lender added. Enter what they actually charged you: the rules and figures behind it differ by country and by lender, so this app does not guess at them.')}
      </p>
      <div class="credit-dialog-fields">
        <label>
          <span>{i18n.t('finances-credit-charge-amount', 'Amount charged')}</span>
          <input type="text" inputmode="decimal" placeholder="0.00" bind:value={chargeAmount} />
        </label>
        <label>
          <span>{i18n.t('finances-credit-charge-note', 'What was it for?')}</span>
          <input type="text" bind:value={chargeNote} />
        </label>
        <label>
          <span>{i18n.t('finances-credit-due-date', 'Due on')}</span>
          <input type="date" bind:value={chargeDate} />
        </label>
      </div>
      <div class="modal-actions">
        <button class="secondary-btn" onclick={() => (charging = null)}>{i18n.t('finances-cancel', 'Cancel')}</button>
        <button class="primary-btn" onclick={saveCharge} disabled={!chargeAmount || !chargeDate}>
          {i18n.t('finances-save', 'Save')}
        </button>
      </div>
    </div>
  </div>
{/if}

<ConfirmDialog
  show={pendingDelete !== null}
  message={i18n.t('finances-credit-delete-confirm', 'Delete this credit? Payments already made stay in your ledger.')}
  danger
  onconfirm={async () => {
    if (pendingDelete) await remove(pendingDelete)
    pendingDelete = null
  }}
  onclose={() => (pendingDelete = null)}
/>

<style>
  .credit-head {
    display: flex; justify-content: space-between; align-items: flex-start; gap: 12px;
  }
  .credit-note {
    margin: 2px 0 0; font-size: 0.78rem; color: var(--text-tertiary); max-width: 52ch; line-height: 1.45;
  }
  .credit-hint {
    margin: -2px 0 0; font-size: 0.74rem; color: var(--text-tertiary); line-height: 1.4;
  }

  .credit-form {
    display: flex; flex-direction: column; gap: 8px; margin: 12px 0;
    padding: 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
  }
  .credit-form input, .credit-form select {
    width: 100%;
    padding: 8px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary); font-size: 0.85rem;
  }
  .credit-form input:focus, .credit-form select:focus {
    border-color: var(--accent); outline: none; box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .credit-form label span { display: block; font-size: 0.78rem; color: var(--text-secondary); margin-bottom: 4px; }
  .credit-pair { display: flex; gap: 8px; }
  .credit-pair label { flex: 1; min-width: 0; }
  .credit-preview {
    display: flex; flex-direction: column; gap: 2px;
    padding: 8px 10px; border-radius: var(--radius-sm); background: var(--glass-active);
    font-size: 0.8rem; color: var(--text-secondary); font-variant-numeric: tabular-nums;
  }
  .credit-extra { color: var(--danger); }
  .credit-actions { display: flex; justify-content: flex-end; gap: 8px; }

  .credit-list { display: flex; flex-direction: column; gap: 12px; margin-top: 12px; }
  .credit-card {
    padding: 14px; border: 1px solid var(--glass-border); border-radius: var(--radius-md);
    background: var(--glass);
  }
  .credit-card.done { opacity: 0.7; }
  .credit-title { display: flex; justify-content: space-between; align-items: flex-start; gap: 10px; }
  .credit-name-wrap { min-width: 0; }
  .credit-name {
    display: block; font-size: 0.95rem; color: var(--text-primary);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .credit-account { font-size: 0.74rem; color: var(--text-tertiary); }

  .credit-bar {
    height: 6px; margin: 10px 0 6px; border-radius: 3px;
    background: var(--glass-active); overflow: hidden;
  }
  .credit-fill { height: 100%; border-radius: 3px; background: var(--accent); transition: width 0.3s ease; }
  .credit-fill.overdue { background: var(--danger); }

  .credit-figures {
    display: flex; justify-content: space-between; gap: 8px;
    font-size: 0.78rem; color: var(--text-secondary); font-variant-numeric: tabular-nums;
  }
  .credit-status-row {
    display: flex; flex-wrap: wrap; gap: 10px; margin-top: 4px;
    font-size: 0.74rem; color: var(--text-tertiary);
  }
  .credit-status.overdue { color: var(--danger); }
  .credit-status.ahead, .credit-status.done { color: var(--success); }
  .credit-next { font-variant-numeric: tabular-nums; }

  .credit-buttons { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 8px; margin-top: 12px; }

  .credit-schedule { margin-top: 12px; border-top: 1px solid var(--glass-border); }
  .credit-inst {
    display: flex; align-items: center; gap: 10px; padding: 7px 0;
    border-bottom: 1px solid var(--glass-border); font-size: 0.78rem;
  }
  .credit-inst:last-child { border-bottom: none; }
  .credit-inst.paid { opacity: 0.6; }
  .credit-inst.charge { background: color-mix(in srgb, var(--danger) 7%, transparent); }
  .credit-inst-num {
    min-width: 64px; flex-shrink: 0;
    color: var(--text-tertiary); font-variant-numeric: tabular-nums;
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .credit-inst-date { flex: 1; color: var(--text-primary); font-variant-numeric: tabular-nums; }
  .credit-inst-amount { color: var(--text-primary); font-variant-numeric: tabular-nums; }
  .credit-inst-state { min-width: 62px; text-align: right; color: var(--text-tertiary); }
  .credit-inst.overdue .credit-inst-state { color: var(--danger); }
  .credit-inst-btn {
    background: none; border: none; padding: 0; cursor: pointer;
    font: inherit; color: var(--accent); transition: opacity 0.15s;
  }
  .credit-inst-btn.danger { color: var(--danger); }
  .credit-inst-btn:hover { opacity: 0.75; }
  .credit-inst-btn:disabled { opacity: 0.4; cursor: default; }

  .credit-table-wrap { margin-top: 12px; overflow-x: auto; }
  .credit-table { width: 100%; border-collapse: collapse; font-size: 0.76rem; }
  .credit-table th, .credit-table td {
    padding: 6px 8px; text-align: right; white-space: nowrap;
    border-bottom: 1px solid var(--glass-border); font-variant-numeric: tabular-nums;
  }
  .credit-table th { color: var(--text-tertiary); font-weight: 500; }
  .credit-table td { color: var(--text-secondary); }
  .credit-table th:first-child, .credit-table td:first-child { text-align: left; }

  /* The surface comes from the shared `.modal`; only the fields are local. */
  .credit-dialog-fields {
    display: flex; flex-direction: column; gap: 12px; margin: 16px 0 4px;
    position: relative; z-index: 10;
  }
  .credit-dialog-fields label span {
    display: block; font-size: 0.78rem; color: var(--text-secondary); margin-bottom: 4px;
  }
  .credit-dialog-fields input {
    width: 100%;
    padding: 9px 12px; border: 1px solid var(--glass-border); border-radius: var(--radius-sm);
    background: var(--select-bg); color: var(--text-primary);
    font-family: inherit; font-size: 0.9rem;
  }
  .credit-dialog-fields input:focus {
    border-color: var(--accent); outline: none; box-shadow: 0 0 0 3px var(--accent-glow);
  }
</style>
