import { invoke } from '@tauri-apps/api/core'
import type { PasswordStrengthResult, VaultExportResult } from '../types'

export async function checkVaultExists(): Promise<boolean> {
  return invoke<boolean>('check_vault_exists')
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

export async function exportVault(): Promise<VaultExportResult> {
  return invoke<VaultExportResult>('export_vault')
}

export async function restoreVault(backupPath: string): Promise<void> {
  return invoke('restore_vault', { backupPath })
}

export async function rollbackRestore(): Promise<void> {
  return invoke('rollback_restore')
}
