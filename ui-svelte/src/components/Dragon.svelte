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

<!-- Decorative dragon silhouette anchored to the bottom-right corner.
     The SVG is used as a CSS mask (not an <img>) so the shape can be painted
     in the live theme accent — an external <img>/<object> SVG cannot be
     recolored from CSS. -->

<div class="dragon-bg" aria-hidden="true"></div>

<style>
  .dragon-bg {
    position: fixed;
    inset: 0;
    z-index: -1; /* above the body backdrop, below the app shell */
    pointer-events: none;
    overflow: hidden;
  }

  .dragon-bg::after {
    content: '';
    position: absolute;
    right: -3%;
    bottom: -5%;
    /* Portrait artwork (~1696x2504, ratio ~0.68): drive the box by height. */
    height: 96vh;
    width: 68vh;
    background-color: var(--accent, #8b5cf6);
    -webkit-mask: url(/assets/bg/Dragon.svg) no-repeat right bottom / contain;
    mask: url(/assets/bg/Dragon.svg) no-repeat right bottom / contain;
    opacity: 0.16;
  }

  /* The accent is darker on white, so ease the opacity for legibility. */
  :global(.light-mode) .dragon-bg::after {
    opacity: 0.08;
  }
</style>
