// Sanctum — a privacy-first personal finance, crypto, and habits vault.
// Copyright (C) 2026  Kyronix
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

export interface AppSettings {
  dark_mode: boolean
  auto_fetch: boolean
  proxy_enabled: boolean
  proxy_url: string
  session_timeout_secs: number
  preferred_currency: string
  preferred_language: string
  sidebar_collapsed: boolean
  login_wallpaper_path: string | null
}

export interface AppInfo {
  version: string
  encryption: string
  storage: string
}
