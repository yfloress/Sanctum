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
  // Three soft purple blobs that drift slowly. The blur is applied once; only
  // `transform` animates, so each blob is cached as a texture and moved on the
  // GPU — no per-frame repaint of the (expensive) blur.
  let paused = $state(false)
  $effect(() => {
    const onVis = () => { paused = document.hidden }
    document.addEventListener('visibilitychange', onVis)
    return () => document.removeEventListener('visibilitychange', onVis)
  })
</script>

<div class="aurora" class:paused aria-hidden="true">
  <span class="blob b1"></span>
  <span class="blob b2"></span>
  <span class="blob b3"></span>
</div>

<style>
  .aurora {
    position: fixed;
    inset: 0;
    z-index: -1;            /* above the body backdrop, below the app shell */
    pointer-events: none;
    overflow: hidden;
  }
  .aurora.paused .blob { animation-play-state: paused; }

  .blob {
    position: absolute;
    border-radius: 50%;
    filter: blur(64px);
    opacity: var(--aurora-opacity, 0.55);
    will-change: transform;
  }
  .b1 {
    width: 46vw; height: 46vw; left: -8vw; top: -10vh;
    background: var(--aurora-1, rgba(168, 85, 247, 0.5));
    animation: float1 26s ease-in-out infinite alternate;
  }
  .b2 {
    width: 40vw; height: 40vw; right: -6vw; top: 16vh;
    background: var(--aurora-2, rgba(99, 102, 241, 0.4));
    animation: float2 32s ease-in-out infinite alternate;
  }
  .b3 {
    width: 38vw; height: 38vw; left: 26vw; bottom: -14vh;
    background: var(--aurora-3, rgba(217, 70, 239, 0.35));
    animation: float3 38s ease-in-out infinite alternate;
  }

  @keyframes float1 { from { transform: translate(0, 0); } to { transform: translate(6vw, 8vh); } }
  @keyframes float2 { from { transform: translate(0, 0); } to { transform: translate(-7vw, 5vh); } }
  @keyframes float3 { from { transform: translate(0, 0); } to { transform: translate(4vw, -6vh); } }
</style>
