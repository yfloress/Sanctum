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

import { invoke } from './ipc'

export interface ExportFolder {
  wallet_id: string
  provider: string
  path: string
}

export interface FolderUpdate {
  inserted: number
  issues: string[]
}

export const listExportFolders = () => invoke<ExportFolder[]>('list_export_folders')
export const chooseExportFolder = (walletId: string, provider: string) =>
  invoke<void>('choose_export_folder', { walletId, provider })
export const removeExportFolder = (walletId: string, provider: string) =>
  invoke<void>('remove_export_folder', { walletId, provider })
export const updateExportFolders = () => invoke<FolderUpdate>('update_export_folders')
