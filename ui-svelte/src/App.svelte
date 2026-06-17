<!-- Sanctum — a privacy-first personal finance and crypto vault.
     Copyright (C) 2026  Kyronix

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
  import { app, type Page } from './lib/stores/app.svelte'
  import { i18n } from './lib/stores/i18n.svelte'
  import { startSessionMonitor } from './lib/stores/session.svelte'
  import Sidebar from './components/Sidebar.svelte'
  import MobileTopBar from './components/MobileTopBar.svelte'
  import StarField from './components/StarField.svelte'
  import Aurora from './components/Aurora.svelte'
  import Toast from './components/Toast.svelte'
  import LoginPage from './pages/LoginPage.svelte'
  import DashboardPage from './pages/DashboardPage.svelte'
  import FinancesPage from './pages/FinancesPage.svelte'
  import CryptoPage from './pages/CryptoPage.svelte'
  import SettingsPage from './pages/SettingsPage.svelte'

  $effect(() => {
    if (app.isLoggedIn) {
      i18n.load()
      return startSessionMonitor()
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
</script>

{#if !app.isLoggedIn}
  <LoginPage />
{:else}
  <div class="shell">
    {#if app.backgroundFx === 'stars'}
      <StarField />
    {:else if app.backgroundFx === 'aurora'}
      <Aurora />
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
    --glass: rgba(255, 255, 255, 0.035);
    --glass-hover: rgba(255, 255, 255, 0.06);
    --glass-active: rgba(255, 255, 255, 0.08);
    --glass-elevated: rgba(255, 255, 255, 0.05);
    --glass-border: rgba(255, 255, 255, 0.08);
    --glass-border-hover: rgba(255, 255, 255, 0.14);
    --glass-blur: blur(16px);
    --glass-blur-heavy: blur(24px);
    --glass-shadow: 0 4px 24px rgba(0, 0, 0, 0.3);
    --glass-shadow-lg: 0 8px 40px rgba(0, 0, 0, 0.45);
    --glass-glow: 0 0 0 1px rgba(255, 255, 255, 0.04) inset;
    --accent: #a855f7;
    --accent-glow: rgba(168, 85, 247, 0.15);
    --accent-hover: #c084fc;
    --success: #4ade80;
    --danger: #f87171;
    --warning: #fbbf24;
    --text-primary: #e8eaed;
    --text-secondary: #9aa0a6;
    --text-tertiary: #5f6368;
    --radius-sm: 8px;
    --radius-md: 12px;
    --radius-lg: 16px;
    --radius-xl: 20px;
    --scrollbar-thumb: rgba(255, 255, 255, 0.12);
    --scrollbar-thumb-hover: rgba(255, 255, 255, 0.22);
    --select-bg: rgba(0, 0, 0, 0.25);
    --select-bg-hover: rgba(0, 0, 0, 0.35);
    --option-bg: #1a1a1a;
    --sidebar-bg: linear-gradient(180deg, #111116 0%, #0d0d11 100%);
    --sidebar-border: rgba(255, 255, 255, 0.06);
    --nav-active-bg: rgba(255, 255, 255, 0.08);
    --nav-active-border: rgba(255, 255, 255, 0.1);
    --danger-bg: rgba(248, 113, 113, 0.08);
    --danger-border: rgba(248, 113, 113, 0.15);
    --modal-bg: linear-gradient(145deg, rgba(26, 26, 31, 0.9) 0%, rgba(20, 20, 24, 0.85) 50%, rgba(17, 17, 21, 0.8) 100%);
    --modal-border: rgba(255, 255, 255, 0.1);
    --modal-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    --accent-bg: rgba(168, 85, 247, 0.2);
    --accent-border: rgba(168, 85, 247, 0.3);
    --text-on-accent: #ffffff;
    --card-bg:
      radial-gradient(circle at 0% 0%, rgba(168, 85, 247, 0.06) 0%, transparent 55%),
      linear-gradient(145deg, rgba(255, 255, 255, 0.05) 0%, rgba(255, 255, 255, 0.02) 100%);
    --card-bg-solid: #0e0c15;
    --card-shadow: 0 4px 24px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.04);
    --card-accent-line: linear-gradient(90deg, transparent 0%, rgba(168, 85, 247, 0.35) 30%, rgba(168, 85, 247, 0.35) 70%, transparent 100%);
  }

  :global(.light-mode) {
    --bg-base: #f8f7fd;
    --bg-gradient: linear-gradient(135deg, #f8f7fd 0%, #f0edff 40%, #e8e3fc 100%);
    --bg-dots: radial-gradient(circle, rgba(124, 58, 237, 0.28) 1px, transparent 1.6px);
    --star-color: rgba(109, 40, 217, 0.95);
    --star-glow: rgba(109, 40, 217, 0.55);
    --diamond-color: rgba(124, 58, 237, 0.20);
    --aurora-1: rgba(139, 92, 246, 0.45);
    --aurora-2: rgba(99, 102, 241, 0.38);
    --aurora-3: rgba(217, 70, 239, 0.32);
    --aurora-opacity: 0.75;
    --glass: rgba(255, 255, 255, 0.65);
    --glass-hover: rgba(147, 51, 234, 0.06);
    --glass-active: rgba(147, 51, 234, 0.1);
    --glass-elevated: rgba(255, 255, 255, 0.8);
    --glass-border: rgba(147, 51, 234, 0.1);
    --glass-border-hover: rgba(147, 51, 234, 0.18);
    --glass-shadow: 0 4px 20px rgba(100, 60, 180, 0.06);
    --glass-shadow-lg: 0 8px 32px rgba(100, 60, 180, 0.1);
    --glass-glow: 0 0 0 1px rgba(147, 51, 234, 0.06) inset;
    --accent: #8b5cf6;
    --accent-glow: rgba(139, 92, 246, 0.15);
    --accent-hover: #7c3aed;
    --success: #16a34a;
    --danger: #dc2626;
    --warning: #d97706;
    --text-primary: #1e1b2e;
    --text-secondary: #4c4665;
    --text-tertiary: #8882a0;
    --scrollbar-thumb: rgba(139, 92, 246, 0.2);
    --scrollbar-thumb-hover: rgba(139, 92, 246, 0.38);
    --select-bg: rgba(255, 255, 255, 0.7);
    --select-bg-hover: rgba(255, 255, 255, 0.9);
    --option-bg: #ffffff;
    --sidebar-bg: linear-gradient(180deg, #f5f3ff 0%, #ede9fe 100%);
    --sidebar-border: rgba(139, 92, 246, 0.1);
    --nav-active-bg: rgba(139, 92, 246, 0.1);
    --nav-active-border: rgba(139, 92, 246, 0.15);
    --danger-bg: rgba(220, 38, 38, 0.08);
    --danger-border: rgba(220, 38, 38, 0.15);
    --modal-bg: linear-gradient(145deg, rgba(255, 255, 255, 0.97) 0%, rgba(250, 248, 255, 0.95) 50%, rgba(245, 243, 255, 0.92) 100%);
    --modal-border: rgba(139, 92, 246, 0.1);
    --modal-shadow: 0 8px 32px rgba(100, 60, 180, 0.1);
    --accent-bg: rgba(139, 92, 246, 0.1);
    --accent-border: rgba(139, 92, 246, 0.2);
    --text-on-accent: #ffffff;
    --card-bg:
      radial-gradient(circle at 0% 0%, rgba(139, 92, 246, 0.06) 0%, transparent 55%),
      linear-gradient(145deg, rgba(255, 255, 255, 0.85) 0%, rgba(255, 255, 255, 0.6) 100%);
    --card-bg-solid: #f2f0fc;
    --card-shadow: 0 4px 20px rgba(100, 60, 180, 0.08), inset 0 1px 0 rgba(255, 255, 255, 0.6);
    --card-accent-line: linear-gradient(90deg, transparent 0%, rgba(139, 92, 246, 0.4) 30%, rgba(139, 92, 246, 0.4) 70%, transparent 100%);
  }

  /* Make white logo visible in light mode */
  :global(.light-mode .logo-icon) {
    filter: brightness(0) saturate(100%) invert(25%) sepia(60%) saturate(4000%) hue-rotate(255deg) brightness(85%) contrast(95%);
  }

  :global(.light-mode .login-logo) {
    filter: brightness(0) saturate(100%) invert(25%) sepia(60%) saturate(4000%) hue-rotate(255deg) brightness(85%) contrast(95%) drop-shadow(0 0 20px rgba(139, 92, 246, 0.25)) !important;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
    background: var(--bg-base);
    background-image: var(--bg-gradient);
    background-attachment: fixed;
    color: var(--text-primary);
    min-height: 100vh;
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
    background: linear-gradient(135deg, rgba(255,255,255,0.04) 0%, rgba(255,255,255,0.015) 100%);
    border: 1px solid var(--glass-border);
    width: fit-content;
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.04), 0 2px 12px rgba(0,0,0,0.2);
    position: relative;
  }

  :global(.light-mode .tab-bar) {
    background: linear-gradient(135deg, rgba(255,255,255,0.7) 0%, rgba(147,51,234,0.04) 100%);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.6), 0 2px 12px rgba(100,60,180,0.06);
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

  :global(.tab-bar button.active) {
    color: var(--text-primary);
    font-weight: 600;
    background: linear-gradient(135deg, var(--accent-bg) 0%, rgba(168,85,247,0.08) 100%);
    border-color: var(--accent-border);
    box-shadow:
      0 0 0 1px var(--accent-border) inset,
      0 2px 10px var(--accent-glow),
      inset 0 1px 0 rgba(255,255,255,0.08);
  }

  :global(.light-mode .tab-bar button.active) {
    background: linear-gradient(135deg, rgba(139,92,246,0.14) 0%, rgba(139,92,246,0.06) 100%);
    box-shadow:
      0 0 0 1px var(--accent-border) inset,
      0 2px 10px var(--accent-glow),
      inset 0 1px 0 rgba(255,255,255,0.5);
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

  .content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    overscroll-behavior-y: contain;
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
