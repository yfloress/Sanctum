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
  import { untrack } from 'svelte'
  import { app, type Page } from './lib/stores/app.svelte'
  import { i18n } from './lib/stores/i18n.svelte'
  import { startSessionMonitor } from './lib/stores/session.svelte'
  import { startShortcuts } from './lib/shortcuts'
  import { autoSyncKraken } from './lib/api/krakenSync'
  import { updateExportFolders } from './lib/api/exportFolders'
  import { startAutoSync } from './lib/autoSync'
  import Sidebar from './components/Sidebar.svelte'
  import MobileTopBar from './components/MobileTopBar.svelte'
  import StarField from './components/StarField.svelte'
  import Aurora from './components/Aurora.svelte'
  import Dragon from './components/Dragon.svelte'
  import Toast from './components/Toast.svelte'
  import SessionLockWarning from './components/SessionLockWarning.svelte'
  import ShortcutsHelp from './components/ShortcutsHelp.svelte'
  import CommandPalette from './components/CommandPalette.svelte'
  import LoginPage from './pages/LoginPage.svelte'
  import DashboardPage from './pages/DashboardPage.svelte'
  import FinancesPage from './pages/FinancesPage.svelte'
  import CryptoPage from './pages/CryptoPage.svelte'
  import SettingsPage from './pages/SettingsPage.svelte'

  let showShortcuts = $state(false)
  let showPalette = $state(false)

  $effect(() => {
    if (app.isLoggedIn) {
      i18n.load()
      let active = true
      void autoSyncKraken().then(result => {
        if (active && result?.inserted) app.showToast('Kraken · ' + result.inserted)
      }).catch(() => { if (active && app.isLoggedIn) app.showToast(i18n.t('crypto-kraken-retry','Review Kraken in History sources'), true) })
      void updateExportFolders().then(result => {
        if (!active) return
        if (result.issues.length) app.showToast(i18n.t('crypto-folders-issues','Some exports need review in Crypto → History sources'), true)
        else if (result.inserted) app.showToast(i18n.t('crypto-folders-count','Imported movements') + ' · ' + result.inserted)
      }).catch(() => {
        if (active && app.isLoggedIn) app.showToast(i18n.t('crypto-folders-failed','Could not check export folders'), true)
      })
      const stopSession = untrack(startSessionMonitor)
      const stopShortcuts = startShortcuts({
        openHelp: () => showShortcuts = true,
        openPalette: () => showPalette = true,
      })
      return () => {
        active = false
        stopSession()
        stopShortcuts()
        showShortcuts = false
        showPalette = false
      }
    }
  })

  // Only `isLoggedIn` and `auto_fetch` are read here; the callbacks read the rest
  // later, so a sync does not restart the loop.
  $effect(() => {
    if (app.isLoggedIn && app.settings?.auto_fetch) {
      return startAutoSync(
        () => app.settings?.prices_updated_at ?? null,
        (at) => app.notePricesSynced(at),
      )
    }
  })

  $effect(() => {
    document.documentElement.dataset.bg = app.backgroundFx
  })

  let lastTheme = app.darkMode
  let themeTimer = 0
  const appMount = Date.now()
  $effect(() => {
    const dark = app.darkMode
    document.documentElement.classList.toggle('light-mode', !dark)
    untrack(() => app.rememberTheme(dark))

    // Crossfade colours only for a genuine user toggle — not the async settings
    // load right after mount (that first flip should be invisible, no startup
    // flash), and not when the user asked the OS to reduce motion.
    const changed = dark !== lastTheme
    lastTheme = dark
    const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    if (changed && Date.now() - appMount > 800 && !reduce) {
      const el = document.documentElement
      el.classList.add('theme-transition')
      clearTimeout(themeTimer)
      themeTimer = window.setTimeout(() => el.classList.remove('theme-transition'), 450)
    }
  })

  let mainEl: HTMLElement | undefined = $state()
  const scrollPositions: Record<Page, number> = {
    dashboard: 0,
    finances: 0,
    crypto: 0,
    settings: 0,
  }
  let trackedPage: Page = app.activePage

  $effect.pre(() => {
    const next = app.activePage
    if (mainEl && next !== trackedPage) {
      scrollPositions[trackedPage] = mainEl.scrollTop
      trackedPage = next
    }
  })

  $effect(() => {
    const page = app.activePage
    if (!mainEl) return
    const target = scrollPositions[page] ?? 0
    requestAnimationFrame(() => {
      if (mainEl) mainEl.scrollTop = target
    })
  })

  function handleScroll() {
    if (mainEl) scrollPositions[app.activePage] = mainEl.scrollTop
  }
  function preventFileNavigation(event: DragEvent) {
    // The file input needs the browser's default drop action to populate its FileList.
    if (event.target instanceof HTMLInputElement && event.target.type === 'file' && !event.target.disabled) return
    if (Array.from(event.dataTransfer?.types ?? []).includes('Files')) event.preventDefault()
  }
</script>

<svelte:window ondragover={preventFileNavigation} ondrop={preventFileNavigation} />

{#if !app.isLoggedIn}
  <LoginPage />
{:else}
  <div class="shell">
    {#if app.backgroundFx === 'stars'}
      <StarField />
    {:else if app.backgroundFx === 'aurora'}
      <Aurora />
    {:else if app.backgroundFx === 'dragon'}
      <Dragon />
    {/if}
    <Sidebar />
    <MobileTopBar />
    <main class="content" bind:this={mainEl} onscroll={handleScroll}>
      {#if app.activePage === 'dashboard'}
        <DashboardPage />
      {:else if app.activePage === 'finances'}
        <FinancesPage />
      {:else if app.activePage === 'crypto'}
        <CryptoPage />
      {:else if app.activePage === 'settings'}
        <SettingsPage />
      {/if}
    </main>
  </div>
{/if}

<SessionLockWarning />

<ShortcutsHelp bind:show={showShortcuts} onclose={() => showShortcuts = false} />

<CommandPalette
  bind:show={showPalette}
  onclose={() => showPalette = false}
  onopenhelp={() => showShortcuts = true}
/>

<Toast />

<style>
  :global(:root) {
    --bg-base: #09090f;
    --bg-gradient: linear-gradient(135deg, #09090f 0%, #0e0b14 40%, #12101a 100%);
    --bg-dots: radial-gradient(circle, rgba(255, 255, 255, 0.05) 1px, transparent 1.6px);
    --star-color: rgba(255, 255, 255, 0.9);
    --star-glow: rgba(255, 255, 255, 0.5);
    --diamond-color: rgba(255, 255, 255, 0.04);
    --aurora-1: rgba(168, 85, 247, 0.50);
    --aurora-2: rgba(99, 102, 241, 0.40);
    --aurora-3: rgba(217, 70, 239, 0.35);
    --aurora-opacity: 0.55;
    /* Overlays tinted with the violet of the page rather than plain white:
       white at a few percent over near-black reads as a flat, cold grey. */
    --glass: rgba(238, 235, 255, 0.035);
    --glass-hover: rgba(238, 235, 255, 0.06);
    --glass-active: rgba(238, 235, 255, 0.08);
    --glass-elevated: rgba(238, 235, 255, 0.05);
    --glass-border: rgba(238, 235, 255, 0.08);
    --glass-border-hover: rgba(238, 235, 255, 0.14);
    --glass-blur: blur(16px);
    --glass-blur-heavy: blur(24px);
    --glass-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
    --glass-shadow-lg: 0 8px 40px rgba(0, 0, 0, 0.45);
    --glass-glow: none;
    --accent: #a855f7;
    --accent-glow: rgba(168, 85, 247, 0.15);
    --accent-hover: #c084fc;
    --success: #4ade80;
    --danger: #f87171;
    --warning: #fbbf24;
    --success-vivid: var(--success);
    --danger-vivid: var(--danger);
    --row-bg: rgba(238, 235, 255, 0.025);
    --row-bg-hover: rgba(238, 235, 255, 0.055);
    --row-border: rgba(238, 235, 255, 0.05);
    /* Radix mauve (dark), the same family as the light theme's neutrals. */
    --text-primary: #eeeef0;
    --text-secondary: #b5b2bc;
    --text-tertiary: #7c7a85;
    --radius-sm: 8px;
    --radius-md: 12px;
    --radius-lg: 16px;
    --radius-xl: 20px;
    --scrollbar-thumb: rgba(255, 255, 255, 0.12);
    --scrollbar-thumb-hover: rgba(255, 255, 255, 0.22);
    --select-bg: rgba(0, 0, 0, 0.25);
    --select-bg-hover: rgba(0, 0, 0, 0.35);
    --option-bg: #16141d;
    --sidebar-bg: #0c0b12;
    --sidebar-border: rgba(238, 235, 255, 0.06);
    --nav-active-bg: rgba(238, 235, 255, 0.08);
    --nav-active-border: rgba(238, 235, 255, 0.1);
    --danger-bg: rgba(248, 113, 113, 0.08);
    --danger-border: rgba(248, 113, 113, 0.15);
    --modal-bg: var(--card-bg-solid);
    --modal-border: rgba(238, 235, 255, 0.1);
    --modal-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    --accent-bg: rgba(168, 85, 247, 0.2);
    --accent-border: rgba(168, 85, 247, 0.3);
    --text-on-accent: #ffffff;
    /* Solid fill for the primary button: deeper than --accent so white text
       clears 4.5:1 on it, which #a855f7 does not. */
    --accent-solid: #7c3aed;
    --accent-solid-hover: #6d28d9;
    --btn-bg: rgba(238, 235, 255, 0.05);
    --btn-bg-hover: rgba(238, 235, 255, 0.09);
    /* Solid and flat: no translucent sheen, no inner highlight. */
    --card-bg: #0e0c15;
    --card-bg-solid: #0e0c15;
    --card-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
    --card-accent-line: linear-gradient(90deg, transparent 0%, rgba(168, 85, 247, 0.35) 30%, rgba(168, 85, 247, 0.35) 70%, transparent 100%);
  }

  /* Light is a daylight theme, not dark inverted: solid white cards over a
     soft mauve page with the dot pattern showing through. --bg-gradient stays a
     single layer, because the dots background repeats its tile size across
     every layer. Neutrals and accent follow Radix Colors (mauve, violet, jade,
     red). Built for bright rooms: body text clears 14:1 and every state colour
     4.5:1 on glass; `-vivid` variants are for large figures and chart fills,
     where 3:1 is enough. */
  :global(.light-mode) {
    --bg-base: #f4f2f7;
    --bg-gradient: linear-gradient(135deg, #f7f6fa 0%, #f4f2f7 50%, #efedf4 100%);
    --bg-dots: radial-gradient(circle, rgba(33, 31, 38, 0.10) 1px, transparent 1.6px);
    --star-color: rgba(101, 80, 185, 0.5);
    --star-glow: rgba(110, 86, 207, 0.22);
    --diamond-color: rgba(33, 31, 38, 0.05);
    --aurora-1: rgba(110, 86, 207, 0.32);
    --aurora-2: rgba(255, 159, 128, 0.30);
    --aurora-3: rgba(217, 119, 242, 0.24);
    --aurora-opacity: 0.45;
    --glass: rgba(255, 255, 255, 0.62);
    --glass-hover: rgba(255, 255, 255, 0.78);
    --glass-active: rgba(255, 255, 255, 0.9);
    --glass-elevated: rgba(255, 255, 255, 0.82);
    --glass-border: rgba(33, 31, 38, 0.08);
    --glass-border-hover: rgba(33, 31, 38, 0.14);
    --glass-blur: blur(20px) saturate(150%);
    --glass-blur-heavy: blur(28px) saturate(160%);
    --glass-shadow: 0 1px 1px rgba(33, 31, 38, 0.03), 0 10px 30px -14px rgba(70, 50, 130, 0.20);
    --glass-shadow-lg: 0 24px 60px -20px rgba(70, 50, 130, 0.28);
    --glass-glow: none;
    --accent: #6e56cf;
    --accent-glow: rgba(110, 86, 207, 0.18);
    --accent-hover: #6550b9;
    --success: #1d775e;
    --success-vivid: #208368;
    --danger: #c12a2f;
    --danger-vivid: #dc3e42;
    --warning: #9a5a00;
    --text-primary: #211f26;
    --text-secondary: #5c5a64;
    --text-tertiary: #65636d;
    --scrollbar-thumb: rgba(33, 31, 38, 0.18);
    --scrollbar-thumb-hover: rgba(33, 31, 38, 0.30);
    --select-bg: rgba(255, 255, 255, 0.75);
    --select-bg-hover: rgba(255, 255, 255, 0.92);
    --option-bg: #ffffff;
    --sidebar-bg: rgba(250, 249, 252, 0.55);
    --sidebar-border: rgba(33, 31, 38, 0.07);
    --nav-active-bg: rgba(255, 255, 255, 0.85);
    --nav-active-border: rgba(110, 86, 207, 0.22);
    --danger-bg: rgba(193, 42, 47, 0.07);
    --danger-border: rgba(193, 42, 47, 0.20);
    --modal-border: rgba(255, 255, 255, 0.9);
    --modal-shadow: 0 30px 80px -24px rgba(50, 35, 100, 0.35), 0 0 0 1px rgba(33, 31, 38, 0.06);
    --accent-bg: rgba(110, 86, 207, 0.10);
    --accent-border: rgba(110, 86, 207, 0.28);
    --text-on-accent: #ffffff;
    --accent-solid: #6e56cf;
    --accent-solid-hover: #6550b9;
    --btn-bg: rgba(255, 255, 255, 0.78);
    --btn-bg-hover: #ffffff;
    --card-bg: #ffffff;
    --row-bg: rgba(33, 31, 38, 0.035);
    --row-bg-hover: rgba(33, 31, 38, 0.06);
    --row-border: rgba(33, 31, 38, 0.05);
    --card-bg-solid: #ffffff;
    --card-shadow: 0 1px 1px rgba(33, 31, 38, 0.03), 0 12px 32px -16px rgba(70, 50, 130, 0.22);
    --card-accent-line: linear-gradient(90deg, transparent 0%, rgba(110, 86, 207, 0.30) 30%, rgba(110, 86, 207, 0.30) 70%, transparent 100%);
  }

  /* Make white logo visible in light mode */
  :global(.light-mode .logo-icon) {
    filter: brightness(0) saturate(100%) invert(25%) sepia(60%) saturate(4000%) hue-rotate(255deg) brightness(85%) contrast(95%);
  }

  :global(.light-mode .login-logo) {
    filter: brightness(0) saturate(100%) invert(25%) sepia(60%) saturate(4000%) hue-rotate(255deg) brightness(85%) contrast(95%) !important;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: 'Instrument Sans Variable', system-ui, -apple-system, sans-serif;
    /* Every digit the same width, so amounts line up in columns and do not
       shift as values change. */
    font-variant-numeric: tabular-nums;
    background: var(--bg-base);
    background-image: var(--bg-gradient);
    background-attachment: fixed;
    color: var(--text-primary);
    min-height: 100vh;
  }

  /* Form controls do not inherit the font by default. */
  :global(button),
  :global(input),
  :global(select),
  :global(textarea) {
    font-family: inherit;
  }

  /* Background designs — chosen from Settings, persisted in localStorage and
     applied via [data-bg] on <html>. 'dots' and 'diamonds' are static CSS
     patterns (zero runtime cost); 'stars' and 'aurora' render as components
     that only mount when selected. */
  :global([data-bg='dots'] body) {
    background-image: var(--bg-dots), var(--bg-gradient);
    background-size: 24px 24px, cover;
    background-position: center, center;
    background-repeat: repeat, no-repeat;
  }
  :global([data-bg='diamonds'] body) {
    background-image:
      repeating-linear-gradient(45deg, var(--diamond-color) 0, var(--diamond-color) 1px, transparent 1px, transparent 22px),
      repeating-linear-gradient(-45deg, var(--diamond-color) 0, var(--diamond-color) 1px, transparent 1px, transparent 22px),
      var(--bg-gradient);
  }

  :global(*) {
    box-sizing: border-box;
  }

  /* Remove the grey tap flash on touch devices for a native feel. */
  :global(html) {
    -webkit-tap-highlight-color: transparent;
  }

  /* Prevent iOS Safari from auto-zooming when focusing an input (font < 16px). */
  @media (max-width: 720px) {
    :global(input),
    :global(select),
    :global(textarea) {
      font-size: 16px;
    }
  }

  :global(::-webkit-scrollbar) {
    width: 4px;
    height: 4px;
  }

  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(::-webkit-scrollbar-thumb) {
    background: var(--scrollbar-thumb);
    border-radius: 99px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: var(--scrollbar-thumb-hover);
  }

  :global(::-webkit-scrollbar-corner) {
    background: transparent;
  }

  :global(*) {
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) transparent;
  }

  :global(select) {
    padding: 8px 12px;
    border: 1px solid var(--glass-border);
    border-radius: 8px;
    background: var(--select-bg);
    color: var(--text-primary);
    font-size: 0.9rem;
    cursor: pointer;
    appearance: none;
    background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23888' stroke-width='2'%3e%3cpath d='M6 9l6 6 6-6'/%3e%3c/svg%3e");
    background-repeat: no-repeat;
    background-position: right 8px center;
    background-size: 20px;
    padding-right: 32px;
    transition: border-color 0.2s, box-shadow 0.2s;
  }

  :global(select:hover) {
    border-color: var(--glass-border-hover);
    background-color: var(--select-bg-hover);
  }

  :global(select:focus) {
    border-color: var(--accent);
    outline: none;
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  :global(select option) {
    background: var(--option-bg);
    color: var(--text-primary);
  }

  :global(.blurred) {
    filter: blur(6px);
    pointer-events: none;
    transition: filter 0.2s ease;
  }

  /* Skeleton shimmer */
  @keyframes shimmer {
    0%   { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
  :global(.skeleton) {
    background: linear-gradient(
      90deg,
      var(--glass) 25%,
      var(--glass-hover) 50%,
      var(--glass) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.8s ease-in-out infinite;
    border-radius: var(--radius-sm);
  }
  :global(.skeleton-row) {
    display: flex; gap: 12px; margin-bottom: 12px;
  }
  :global(.skeleton-grid) {
    display: grid; gap: 12px; margin-bottom: 16px;
  }

  :global(.glass-btn) {
    padding: 8px 16px;
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    background: var(--glass);
    color: var(--text-primary);
    font-size: 0.85rem;
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s, box-shadow 0.2s;
  }

  :global(.glass-btn:hover) {
    background: var(--glass-hover);
    border-color: var(--glass-border-hover);
    box-shadow: var(--glass-shadow);
  }

  :global(.glass-btn:active) {
    background: var(--glass-active);
    transform: scale(0.98);
  }

  :global(.tab-bar) {
    display: flex;
    gap: 2px;
    padding: 5px;
    border-radius: 14px;
    background: var(--card-bg-solid);
    border: 1px solid var(--glass-border);
    width: fit-content;
    box-shadow: 0 2px 12px rgba(0,0,0,0.2);
    position: relative;
  }

  :global(.light-mode .tab-bar) {
    background: var(--glass);
    box-shadow: 0 8px 24px -14px rgba(70,50,130,0.22);
  }

  :global(.tab-bar button) {
    padding: 8px 20px;
    border: 1px solid transparent;
    border-radius: 10px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.85rem;
    font-weight: 500;
    letter-spacing: 0.01em;
    cursor: pointer;
    transition: color 0.2s, background 0.25s cubic-bezier(0.16, 1, 0.3, 1),
                border-color 0.25s, box-shadow 0.25s, transform 0.15s;
    position: relative;
  }

  :global(.tab-bar button:hover:not(.active)) {
    color: var(--text-primary);
    background: var(--glass-hover);
  }

  :global(.tab-bar button:active) {
    transform: scale(0.97);
  }

  /* Flat on purpose: the purple tint and its border mark the tab, no glow. */
  :global(.tab-bar button.active) {
    color: var(--text-primary);
    font-weight: 600;
    background: var(--accent-bg);
    border-color: var(--accent-border);
  }

  :global(.light-mode .tab-bar button.active) {
    background: rgba(255,255,255,0.92);
  }

  .shell {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    /* Own stacking context so the z-index:-1 background layer (StarField /
       Aurora) sits above the body's opaque background instead of being
       painted behind it in the root context. */
    isolation: isolate;
  }

  /* Only on touch screens, where it stops the pull-to-refresh bounce. On the
     desktop it buys nothing, and WebKitGTK has had bugs where it leaves the
     mouse wheel unable to scroll. */
  @media (pointer: coarse) {
    .content { overscroll-behavior-y: contain; }
  }

  .content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
  }

  /* ── Mobile: leave room for the fixed bottom navigation bar ── */
  @media (max-width: 720px) {
    .content {
      padding-top: calc(52px + env(safe-area-inset-top, 0px));
      padding-bottom: calc(60px + env(safe-area-inset-bottom, 0px));
    }
  }

  /* ── Mobile: tab bars span the width and scroll instead of overflowing ── */
  @media (max-width: 480px) {
    :global(.tab-bar) {
      width: 100%;
      overflow-x: auto;
      scrollbar-width: none;
    }
    :global(.tab-bar)::-webkit-scrollbar {
      display: none;
    }
    :global(.tab-bar button) {
      padding: 8px 14px;
      white-space: nowrap;
      flex: 1 0 auto;
    }
  }
</style>
