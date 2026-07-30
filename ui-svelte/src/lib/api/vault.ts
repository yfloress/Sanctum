// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

import { invoke } from '@tauri-apps/api/core'
import type { PasswordStrengthResult, VaultStatus } from '../types'

export async function checkVaultExists(): Promise<boolean> {
  const status = await invoke<VaultStatus>('check_vault_exists')
  return status.exists
}

export async function createVault(password: string): Promise<void> {
  return invoke('create_vault', { password })
}

export async function unlockVault(password: string): Promise<void> {
  return invoke('unlock_vault', { password })
}

export async function lockVault(): Promise<void> {
  return invoke('lock_vault')
}

export async function checkPasswordStrength(password: string): Promise<PasswordStrengthResult> {
  return invoke<PasswordStrengthResult>('check_password_strength', { password })
}

export async function exportVault(path: string): Promise<void> {
  return invoke('export_vault', { path })
}

export async function restoreVault(backup_path: string): Promise<void> {
  return invoke('restore_vault', { backup_path })
}

export async function rollbackRestore(): Promise<void> {
  return invoke('rollback_restore')
}

/**
 * Changes the master password, re-encrypting the vault.
 *
 * Returns the path of the rollback copy written first, which stays encrypted
 * with the OLD password.
 */
export async function changeVaultPassword(
  currentPassword: string,
  newPassword: string
): Promise<string> {
  return invoke<string>('change_vault_password', { currentPassword, newPassword })
}
