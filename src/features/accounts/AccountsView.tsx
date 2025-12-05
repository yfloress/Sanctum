/**
 * Accounts View Component
 *
 * Displays FIAT accounts with balances, create/edit forms, and transfer functionality.
 * Consumes state directly from Zustand stores - no props needed.
 */

import { useState, type FormEvent } from "react";
import {
  useAccountStore,
  useAccounts,
  useAccountBalances,
  useAccountLoading,
  useAccountForm,
  useAccountToEdit,
  ACCOUNT_TYPES,
  ACCOUNT_COLORS,
} from "../../stores/accountStore.ts";
import { formatAmount } from "../../utils/index.ts";

type ModalType = "none" | "account" | "transfer";

export function AccountsView() {
  // ==================== Store State ====================
  const accounts = useAccounts();
  const balances = useAccountBalances();
  const isLoading = useAccountLoading();
  const form = useAccountForm();
  const accountToEdit = useAccountToEdit();

  // ==================== Store Actions ====================
  const {
    setFormField,
    resetForm,
    createAccount,
    updateAccount,
    archiveAccount,
    setAccountToEdit,
    transfer,
    getTotalNetWorth,
  } = useAccountStore();

  // ==================== Local State ====================
  const [modalType, setModalType] = useState<ModalType>("none");
  const [transferForm, setTransferForm] = useState({
    fromAccountId: "",
    toAccountId: "",
    amount: "",
    description: "",
    date: new Date().toISOString().split("T")[0],
  });

  // ==================== Computed ====================
  const activeAccounts = accounts.filter((acc) => !acc.is_archived);
  const archivedAccounts = accounts.filter((acc) => acc.is_archived);
  const netWorth = getTotalNetWorth();

  // ==================== Helpers ====================
  const getBalanceForAccount = (accountId: string) => {
    const balance = balances.find((b) => b.account_id === accountId);
    return balance?.current_balance ?? 0;
  };

  const getAccountTypeInfo = (type: string) => {
    return ACCOUNT_TYPES.find((t) => t.value === type) || ACCOUNT_TYPES[4]; // Default to "other"
  };

  // ==================== Modal Handlers ====================
  const openCreateModal = () => {
    resetForm();
    setModalType("account");
  };

  const openEditModal = (account: typeof accounts[0]) => {
    setAccountToEdit(account);
    setModalType("account");
  };

  const openTransferModal = () => {
    setTransferForm({
      fromAccountId: activeAccounts[0]?.id || "",
      toAccountId: activeAccounts[1]?.id || "",
      amount: "",
      description: "",
      date: new Date().toISOString().split("T")[0],
    });
    setModalType("transfer");
  };

  const closeModal = () => {
    setModalType("none");
    resetForm();
  };

  // ==================== Form Handlers ====================
  const handleAccountSubmit = async (e: FormEvent) => {
    e.preventDefault();

    let success: boolean;
    if (accountToEdit) {
      success = await updateAccount(accountToEdit.id, form);
    } else {
      success = await createAccount(form);
    }

    if (success) {
      closeModal();
    }
  };

  const handleTransferSubmit = async (e: FormEvent) => {
    e.preventDefault();

    const success = await transfer(
      transferForm.fromAccountId,
      transferForm.toAccountId,
      parseFloat(transferForm.amount) || 0,
      transferForm.description,
      transferForm.date
    );

    if (success) {
      closeModal();
    }
  };

  const handleArchive = async (accountId: string) => {
    await archiveAccount(accountId);
  };

  // ==================== Render ====================
  return (
    <div className="accounts-page">
      <div className="accounts-header">
        <div className="accounts-title-section">
          <h1 className="page-title">Accounts</h1>
          <div className="net-worth-badge">
            <span className="net-worth-label">Net Worth</span>
            <span className={`net-worth-value ${netWorth >= 0 ? "positive" : "negative"}`}>
              ${formatAmount(netWorth)}
            </span>
          </div>
        </div>
        <div className="accounts-actions">
          <button
            type="button"
            className="btn-secondary"
            onClick={openTransferModal}
            disabled={isLoading || activeAccounts.length < 2}
            title={activeAccounts.length < 2 ? "Need at least 2 accounts" : "Transfer between accounts"}
          >
            <span>↔️</span> Transfer
          </button>
          <button
            type="button"
            className="btn-primary"
            onClick={openCreateModal}
            disabled={isLoading}
          >
            <span>+</span> New Account
          </button>
        </div>
      </div>

      {/* Active Accounts Grid */}
      <div className="accounts-section">
        <h2 className="section-title">Active Accounts</h2>
        {activeAccounts.length === 0 ? (
          <div className="accounts-empty">
            <span className="empty-icon">🏦</span>
            <h3>No accounts yet</h3>
            <p>Create your first account to start tracking your finances.</p>
            <button type="button" className="btn-primary" onClick={openCreateModal}>
              Create Account
            </button>
          </div>
        ) : (
          <div className="accounts-grid">
            {activeAccounts.map((account) => {
              const typeInfo = getAccountTypeInfo(account.type);
              const currentBalance = getBalanceForAccount(account.id);

              return (
                <div
                  key={account.id}
                  className="account-card"
                  style={{ borderLeftColor: account.color }}
                >
                  <div className="account-card-header">
                    <div className="account-icon" style={{ backgroundColor: account.color }}>
                      {account.icon || typeInfo.icon}
                    </div>
                    <div className="account-info">
                      <h3 className="account-name">{account.name}</h3>
                      <span className="account-type">{typeInfo.label}</span>
                    </div>
                    <div className="account-card-actions">
                      <button
                        type="button"
                        className="account-action-btn"
                        onClick={() => openEditModal(account)}
                        title="Edit account"
                      >
                        ✏️
                      </button>
                      <button
                        type="button"
                        className="account-action-btn delete"
                        onClick={() => handleArchive(account.id)}
                        title="Archive account"
                      >
                        📥
                      </button>
                    </div>
                  </div>
                  <div className="account-balance">
                    <span className="balance-label">Current Balance</span>
                    <span className={`balance-amount ${currentBalance >= 0 ? "positive" : "negative"}`}>
                      {currentBalance < 0 ? "-" : ""}${formatAmount(Math.abs(currentBalance))}
                    </span>
                  </div>
                  <div className="account-currency">{account.currency}</div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Archived Accounts */}
      {archivedAccounts.length > 0 && (
        <div className="accounts-section archived-section">
          <h2 className="section-title">Archived Accounts</h2>
          <div className="accounts-grid">
            {archivedAccounts.map((account) => {
              const typeInfo = getAccountTypeInfo(account.type);
              const currentBalance = getBalanceForAccount(account.id);

              return (
                <div
                  key={account.id}
                  className="account-card archived"
                  style={{ borderLeftColor: account.color }}
                >
                  <div className="account-card-header">
                    <div className="account-icon" style={{ backgroundColor: account.color, opacity: 0.5 }}>
                      {account.icon || typeInfo.icon}
                    </div>
                    <div className="account-info">
                      <h3 className="account-name">{account.name}</h3>
                      <span className="account-type">{typeInfo.label} (Archived)</span>
                    </div>
                  </div>
                  <div className="account-balance">
                    <span className="balance-label">Final Balance</span>
                    <span className="balance-amount">${formatAmount(currentBalance)}</span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Account Modal (Create/Edit) */}
      {modalType === "account" && (
        <div className="modal-overlay" onClick={closeModal}>
          <div className="modal-card account-modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <span className="modal-icon">🏦</span>
              <h2>{accountToEdit ? "Edit Account" : "New Account"}</h2>
            </div>
            <form onSubmit={handleAccountSubmit}>
              <div className="modal-body">
                <div className="form-group">
                  <label htmlFor="account-name">Account Name</label>
                  <input
                    id="account-name"
                    type="text"
                    value={form.name}
                    onChange={(e) => setFormField("name", e.target.value)}
                    placeholder="e.g., Main Checking"
                    disabled={isLoading}
                    autoFocus
                  />
                </div>

                <div className="form-row">
                  <div className="form-group">
                    <label htmlFor="account-type">Type</label>
                    <select
                      id="account-type"
                      value={form.type}
                      onChange={(e) => {
                        setFormField("type", e.target.value);
                        const typeInfo = ACCOUNT_TYPES.find((t) => t.value === e.target.value);
                        if (typeInfo) {
                          setFormField("icon", typeInfo.icon);
                        }
                      }}
                      disabled={isLoading}
                    >
                      {ACCOUNT_TYPES.map((type) => (
                        <option key={type.value} value={type.value}>
                          {type.icon} {type.label}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div className="form-group">
                    <label htmlFor="account-currency">Currency</label>
                    <select
                      id="account-currency"
                      value={form.currency}
                      onChange={(e) => setFormField("currency", e.target.value)}
                      disabled={isLoading}
                    >
                      <option value="USD">USD ($)</option>
                      <option value="EUR">EUR (€)</option>
                      <option value="GBP">GBP (£)</option>
                      <option value="MXN">MXN ($)</option>
                    </select>
                  </div>
                </div>

                <div className="form-group">
                  <label htmlFor="initial-balance">
                    {accountToEdit ? "Initial Balance" : "Starting Balance"} ($)
                  </label>
                  <input
                    id="initial-balance"
                    type="number"
                    step="0.01"
                    value={form.initial_balance}
                    onChange={(e) => setFormField("initial_balance", e.target.value)}
                    placeholder="0.00"
                    disabled={isLoading}
                  />
                </div>

                <div className="form-group">
                  <label>Color</label>
                  <div className="color-picker">
                    {ACCOUNT_COLORS.map((color) => (
                      <button
                        key={color}
                        type="button"
                        className={`color-option ${form.color === color ? "selected" : ""}`}
                        style={{ backgroundColor: color }}
                        onClick={() => setFormField("color", color)}
                        disabled={isLoading}
                        aria-label={`Select color ${color}`}
                      />
                    ))}
                  </div>
                </div>
              </div>

              <div className="modal-actions">
                <button
                  type="button"
                  className="btn-secondary"
                  onClick={closeModal}
                  disabled={isLoading}
                >
                  Cancel
                </button>
                <button type="submit" className="btn-primary" disabled={isLoading}>
                  {isLoading ? "Saving..." : accountToEdit ? "Update" : "Create"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Transfer Modal */}
      {modalType === "transfer" && (
        <div className="modal-overlay" onClick={closeModal}>
          <div className="modal-card transfer-modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <span className="modal-icon">↔️</span>
              <h2>Transfer Funds</h2>
            </div>
            <form onSubmit={handleTransferSubmit}>
              <div className="modal-body">
                <div className="form-group">
                  <label htmlFor="from-account">From Account</label>
                  <select
                    id="from-account"
                    value={transferForm.fromAccountId}
                    onChange={(e) =>
                      setTransferForm({ ...transferForm, fromAccountId: e.target.value })
                    }
                    disabled={isLoading}
                  >
                    {activeAccounts.map((acc) => (
                      <option key={acc.id} value={acc.id}>
                        {acc.name} (${formatAmount(getBalanceForAccount(acc.id))})
                      </option>
                    ))}
                  </select>
                </div>

                <div className="transfer-arrow">⬇️</div>

                <div className="form-group">
                  <label htmlFor="to-account">To Account</label>
                  <select
                    id="to-account"
                    value={transferForm.toAccountId}
                    onChange={(e) =>
                      setTransferForm({ ...transferForm, toAccountId: e.target.value })
                    }
                    disabled={isLoading}
                  >
                    {activeAccounts
                      .filter((acc) => acc.id !== transferForm.fromAccountId)
                      .map((acc) => (
                        <option key={acc.id} value={acc.id}>
                          {acc.name} (${formatAmount(getBalanceForAccount(acc.id))})
                        </option>
                      ))}
                  </select>
                </div>

                <div className="form-row">
                  <div className="form-group">
                    <label htmlFor="transfer-amount">Amount ($)</label>
                    <input
                      id="transfer-amount"
                      type="number"
                      step="0.01"
                      min="0.01"
                      value={transferForm.amount}
                      onChange={(e) =>
                        setTransferForm({ ...transferForm, amount: e.target.value })
                      }
                      placeholder="0.00"
                      disabled={isLoading}
                      autoFocus
                    />
                  </div>

                  <div className="form-group">
                    <label htmlFor="transfer-date">Date</label>
                    <input
                      id="transfer-date"
                      type="date"
                      value={transferForm.date}
                      onChange={(e) =>
                        setTransferForm({ ...transferForm, date: e.target.value })
                      }
                      disabled={isLoading}
                    />
                  </div>
                </div>

                <div className="form-group">
                  <label htmlFor="transfer-description">Description (optional)</label>
                  <input
                    id="transfer-description"
                    type="text"
                    value={transferForm.description}
                    onChange={(e) =>
                      setTransferForm({ ...transferForm, description: e.target.value })
                    }
                    placeholder="e.g., Monthly savings transfer"
                    disabled={isLoading}
                  />
                </div>
              </div>

              <div className="modal-actions">
                <button
                  type="button"
                  className="btn-secondary"
                  onClick={closeModal}
                  disabled={isLoading}
                >
                  Cancel
                </button>
                <button type="submit" className="btn-primary" disabled={isLoading}>
                  {isLoading ? "Transferring..." : "Transfer"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
