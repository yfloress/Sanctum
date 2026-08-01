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

// The commands a page owns, declared here rather than inside the page itself.
//
// The palette has to list them from anywhere, including while their page is
// unmounted and its handlers do not exist. So the name lives here and the page
// contributes only the body, keyed by id, once it is on screen: running one
// from elsewhere navigates first and the page picks it up as it appears.

import type { Page } from './stores/app.svelte'

export interface CommandSpec {
  id: string
  /** Fluent key, and the English text used when the key is missing. */
  key: string
  fallback: string
  /** The page that supplies the handler, and the one the palette navigates to. */
  page: Page
}

export const PAGE_COMMANDS: CommandSpec[] = [
  { id: 'fin-tab-overview', key: 'finances-tab-overview', fallback: 'Overview', page: 'finances' },
  { id: 'fin-tab-activity', key: 'finances-tab-activity', fallback: 'Activity', page: 'finances' },
  { id: 'fin-tab-accounts', key: 'finances-tab-accounts', fallback: 'Accounts', page: 'finances' },
  { id: 'fin-tab-settings', key: 'finances-tab-settings', fallback: 'Settings', page: 'finances' },
  { id: 'fin-new-account', key: 'finances-new-account', fallback: 'New Account', page: 'finances' },
  { id: 'fin-transfer', key: 'finances-transfer', fallback: 'Transfer', page: 'finances' },

  { id: 'crypto-tab-portfolio', key: 'crypto-tab-portfolio', fallback: 'Portfolio', page: 'crypto' },
  { id: 'crypto-tab-wallets', key: 'crypto-tab-wallets', fallback: 'Wallets', page: 'crypto' },
  { id: 'crypto-tab-activity', key: 'crypto-tab-activity', fallback: 'Activity', page: 'crypto' },
  { id: 'crypto-tab-tax', key: 'crypto-tab-tax', fallback: 'Tax', page: 'crypto' },
  { id: 'crypto-add-wallet', key: 'crypto-add-wallet', fallback: 'Add Wallet', page: 'crypto' },
  { id: 'crypto-sync', key: 'crypto-sync-prices', fallback: 'Sync prices', page: 'crypto' },
  { id: 'crypto-ticker', key: 'crypto-configure-ticker', fallback: 'Configure ticker', page: 'crypto' },

  { id: 'set-export-vault', key: 'settings-export-vault', fallback: 'Export Vault', page: 'settings' },
  {
    id: 'set-export-csv',
    key: 'settings-export-transactions',
    fallback: 'Export Transactions',
    page: 'settings',
  },
]
