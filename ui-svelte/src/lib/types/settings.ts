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
