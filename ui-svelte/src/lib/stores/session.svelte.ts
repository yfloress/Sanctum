import { app } from './app.svelte'
import * as vaultApi from '../api/vault'

let timer: ReturnType<typeof setInterval> | null = null
let lastActivity = Date.now()

function resetActivity() {
  lastActivity = Date.now()
}

async function checkTimeout() {
  if (!app.isLoggedIn || !app.settings) return

  const timeoutSecs = app.settings.session_timeout_secs
  if (timeoutSecs <= 0) return

  const elapsed = (Date.now() - lastActivity) / 1000
  if (elapsed >= timeoutSecs) {
    await vaultApi.lockVault()
    app.logout()
  }
}

export function startSessionMonitor() {
  const events = ['mousedown', 'keydown', 'touchstart', 'scroll'] as const
  events.forEach(e => document.addEventListener(e, resetActivity, { passive: true }))

  timer = setInterval(checkTimeout, 10_000)

  return () => {
    events.forEach(e => document.removeEventListener(e, resetActivity))
    if (timer) { clearInterval(timer); timer = null }
  }
}
