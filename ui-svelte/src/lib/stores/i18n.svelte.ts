import * as settingsApi from '../api/settings'

class I18nStore {
  strings = $state<Record<string, string>>({})
  loaded = $state(false)

  async load() {
    try {
      this.strings = await settingsApi.getTranslations()
      this.loaded = true
    } catch (e) {
      console.error('Failed to load translations:', e)
    }
  }

  t(key: string, fallback?: string): string {
    return this.strings[key] ?? fallback ?? key
  }
}

export const i18n = new I18nStore()
