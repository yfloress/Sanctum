<!-- Sanctum — a privacy-first personal finance, crypto, and habits vault.
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
  // Sparse, twinkling starfield. Built once on mount — positions, sizes and
  // twinkle timing are randomised but fixed for the session, so the only
  // per-frame work is GPU-composited opacity/transform on a handful of nodes.
  const STAR_COUNT = 90

  type Star = {
    x: number; y: number; size: number; delay: number; dur: number; bright: boolean
  }

  const stars: Star[] = Array.from({ length: STAR_COUNT }, () => {
    const size = 1 + Math.random() * 1.6
    return {
      x: Math.random() * 100,
      y: Math.random() * 100,
      size,
      delay: Math.random() * 6,
      dur: 2.8 + Math.random() * 3.5,
      bright: size > 2.1,
    }
  })

  // Stop animating while the window is hidden — no point spending battery on a
  // backdrop nobody can see.
  let paused = $state(false)
  $effect(() => {
    const onVis = () => { paused = document.hidden }
    document.addEventListener('visibilitychange', onVis)
    return () => document.removeEventListener('visibilitychange', onVis)
  })
</script>

<div class="starfield" class:paused aria-hidden="true">
  {#each stars as s}
    <span
      class="star"
      class:bright={s.bright}
      style="left:{s.x}%; top:{s.y}%; width:{s.size}px; height:{s.size}px;
             animation-delay:{s.delay}s; animation-duration:{s.dur}s;"
    ></span>
  {/each}
</div>

<style>
  .starfield {
    position: fixed;
    inset: 0;
    z-index: -1;            /* above the body backdrop, below the app shell */
    pointer-events: none;
    overflow: hidden;
    animation: drift 90s linear infinite alternate;
  }
  .starfield.paused,
  .starfield.paused .star {
    animation-play-state: paused;
  }

  .star {
    position: absolute;
    border-radius: 50%;
    background: var(--star-color, rgba(255, 255, 255, 0.9));
    opacity: 0.12;
    animation-name: twinkle;
    animation-timing-function: ease-in-out;
    animation-iteration-count: infinite;
    will-change: opacity;
  }
  .star.bright {
    box-shadow: 0 0 4px 1px var(--star-glow, rgba(255, 255, 255, 0.5));
  }

  @keyframes twinkle {
    0%, 100% { opacity: 0.1; }
    50%      { opacity: 0.85; }
  }
  @keyframes drift {
    from { transform: translateY(0); }
    to   { transform: translateY(-22px); }
  }
</style>
