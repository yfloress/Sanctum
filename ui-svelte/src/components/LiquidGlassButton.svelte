<script>
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';

	/** @type {any} */
	let { contrast = 'dark', text = 'Button', onclick = () => {} } = $props();

	let rootElement = $state();
	// declare css variables for web-components
	onMount(() => {
		rootElement.style.setProperty('--anim--hover-time', `400ms`);
		rootElement.style.setProperty('--anim--hover-ease', `cubic-bezier(0.25, 1, 0.5, 1)`);
		rootElement.style.setProperty('--angle-1', ` -75deg`);
	});

	// update css roundess
	$effect(() => {
		if (rootElement) {
			rootElement.style.setProperty('--roundness', `12px`);
		}
	});

	let isHovering = $state(false);
	function toggleHover() {
		isHovering = !isHovering;
	}
	let isDark = $derived(contrast == 'dark' || contrast == 'dark-contrast');

	let textColor = $state('text-white');
	$effect(() => {
		switch (contrast) {
			case 'light-contrast':
				textColor = 'text-black';
				break;
			case 'dark-contrast':
				textColor = 'text-black/50';
				break;
			default:
				textColor = 'text-white';
				break;
		}
	});

	const accentColor = '#4f9cf7'; // Sanctum blue
</script>

<div
	bind:this={rootElement}
	onmouseenter={toggleHover}
	onmouseleave={toggleHover}
	role="presentation"
	class="button-wrap relative overscroll-contain">

	<button
		class="glassy-button overflow-hidden {isDark ? 'dark-glassy-button' : 'light-glassy-button'}"
		{onclick}>
		<span class="{textColor} text-nowrap" style="font-weight: 600; font-size: 0.95rem; padding-inline: 1rem; padding-block: 0.5rem;">
			{text}
		</span>
	</button>
	<div class="button-shadow {isDark ? 'dark-shadow' : 'light-shadow'}"></div>
	<div class="glass-filter" style="border-radius: 12px"></div>
</div>

<svg style="display: none;border-radius: 12px">
	<filter id="lg-dist" x="0%" y="0%" width="100%" height="100%">
		<feTurbulence type="fractalNoise" baseFrequency="0.008 0.008" numOctaves="2" seed="92" result="noise" />
		<feGaussianBlur in="noise" stdDeviation="2" result="blurred" />
		<feDisplacementMap in="SourceGraphic" in2="blurred" scale="230" xChannelSelector="R" yChannelSelector="G" />
	</filter>
</svg>

<style>
	/* design inspired by https://codepen.io/odibixie/pen/vEYEWQR & danilofiumi/liquid-glass-svelte */
	@property --angle-1 {
		syntax: '<angle>';
		inherits: false;
		initial-value: -75deg;
	}

	@property --angle-2 {
		syntax: '<angle>';
		inherits: false;
		initial-value: -45deg;
	}

	:root {
		--global--size: clamp(2rem, 4vw, 5rem);
		--anim--hover-time: 400ms;
		--anim--hover-ease: cubic-bezier(0.25, 1, 0.5, 1);
	}

	filter,
	svg {
		border-radius: var(--roundness);
		overflow: hidden;
	}

	.button-wrap {
		position: relative;
		overflow: hidden;
		border-radius: var(--roundness);
		background: transparent;
		pointer-events: none;
		transition: all var(--anim--hover-time) var(--anim--hover-ease);
		width: fit-content;
	}

	.glass-filter {
		position: absolute;
		inset: 0;
		z-index: 0;
		-webkit-backdrop-filter: blur(4px);
		backdrop-filter: blur(4px);
		filter: url(#lg-dist) saturate(150%);
		isolation: isolate;
	}

	.button-shadow {
		--shadow-cuttoff-fix: 2em;
		position: absolute;
		width: calc(100% + var(--shadow-cuttoff-fix));
		height: calc(100% + var(--shadow-cuttoff-fix));
		top: calc(0% - var(--shadow-cuttoff-fix) / 2);
		left: calc(0% - var(--shadow-cuttoff-fix) / 2);
		filter: blur(clamp(2px, 0.125em, 12px));
		-webkit-filter: blur(clamp(2px, 0.125em, 12px));
		overflow: visible;
		pointer-events: none;
	}

	.button-shadow::after {
		content: '';
		position: absolute;
		z-index: 0;
		inset: 0;
		border-radius: var(--roundness);
		width: calc(100% - var(--shadow-cuttoff-fix) - 0.25em);
		height: calc(100% - var(--shadow-cuttoff-fix) - 0.25em);
		top: calc(var(--shadow-cuttoff-fix) - 0.5em);
		left: calc(var(--shadow-cuttoff-fix) - 0.875em);
		padding: 0.125em;
		box-sizing: border-box;
		mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0);
		mask-composite: exclude;
		transition: all var(--anim--hover-time) var(--anim--hover-ease);
		overflow: visible;
		opacity: 1;
	}

	.glassy-button {
		--border-width: clamp(1px, 0.0625em, 4px);
		all: unset;
		cursor: pointer;
		position: relative;
		pointer-events: auto;
		z-index: 3;

		border-radius: var(--roundness);

		backdrop-filter: blur(clamp(1px, 0.125em, 4px));
		-webkit-backdrop-filter: blur(clamp(1px, 0.125em, 4px));
		transition: all var(--anim--hover-time) var(--anim--hover-ease);
	}

	.glassy-button:hover {
		transform: scale(0.975);
		backdrop-filter: blur(0.01em);
		-webkit-backdrop-filter: blur(0.01em);
	}

	.glassy-button span {
		position: relative;
		display: block;
		user-select: none;
		-webkit-user-select: none;
		-moz-user-select: none;
		-webkit-font-smoothing: antialiased;
		-moz-osx-font-smoothing: grayscale;
		transition: all var(--anim--hover-time) var(--anim--hover-ease);
	}

	.glassy-button span::after {
		content: '';
		display: block;
		position: absolute;
		z-index: 1;
		width: calc(100% - var(--border-width));
		height: calc(100% - var(--border-width));
		top: calc(0% + var(--border-width) / 2);
		left: calc(0% + var(--border-width) / 2);
		box-sizing: border-box;
		border-radius: var(--roundness);
		overflow: clip;
		z-index: 3;
		mix-blend-mode: screen;
		pointer-events: none;
		background-size: 200% 200%;
		background-position: 0% 50%;
		background-repeat: no-repeat;
		transition: background-position calc(var(--anim--hover-time) * 1.25) var(--anim--hover-ease),
			--angle-2 calc(var(--anim--hover-time) * 1.25) var(--anim--hover-ease);
	}

	.glassy-button:hover span::after {
		background-position: 25% 50%;
	}

	.glassy-button:active span::after {
		background-position: 50% 15%;
		--angle-2: -15deg;
	}

	.glassy-button::after {
		content: '';
		position: absolute;
		z-index: 1;
		inset: 0;
		border-radius: var(--roundness);
		width: calc(100% + var(--border-width));
		height: calc(100% + var(--border-width));
		top: calc(0% - var(--border-width) / 2);
		left: calc(0% - var(--border-width) / 2);
		padding: var(--border-width);
		box-sizing: border-box;
		mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0);
		mask-composite: exclude;
		transition: all var(--anim--hover-time) var(--anim--hover-ease), --angle-1 500ms ease;
	}

	.glassy-button:hover::after {
		--angle-1: -125deg;
	}

	.glassy-button:active::after {
		--angle-1: -75deg;
	}

	.button-wrap:has(.glassy-button:hover) .button-shadow {
		filter: blur(clamp(2px, 0.0625em, 6px));
		-webkit-filter: blur(clamp(2px, 0.0625em, 6px));
		transition: filter var(--anim--hover-time) var(--anim--hover-ease);
	}

	.button-wrap:has(.glassy-button:hover) .button-shadow::after {
		top: calc(var(--shadow-cuttoff-fix) - 0.875em);
		opacity: 1;
	}

	.button-wrap:has(.glassy-button:active) {
		transform: rotate3d(1, 0, 0, 25deg);
	}

	.button-wrap:has(.glassy-button:active) .button-shadow {
		filter: blur(clamp(2px, 0.125em, 12px));
		-webkit-filter: blur(clamp(2px, 0.125em, 12px));
	}

	.button-wrap:has(.glassy-button:active) .button-shadow::after {
		top: calc(var(--shadow-cuttoff-fix) - 0.5em);
		opacity: 0.75;
	}

	.dark-shadow::after {
		background: linear-gradient(180deg, rgba(254, 254, 254, 0.2), rgba(254, 254, 254, 0.1));
	}

	.light-shadow::after {
		background: linear-gradient(180deg, rgba(0, 0, 0, 0.2), rgba(0, 0, 0, 0.1));
	}

	.light-glassy-button {
		-webkit-tap-highlight-color: rgba(0, 0, 0, 0);
		background: linear-gradient(
			-75deg,
			rgba(255, 255, 255, 0.05),
			rgba(255, 255, 255, 0.2),
			rgba(255, 255, 255, 0.05)
		);
		box-shadow:
			inset 0 0.125em 0.125em rgba(0, 0, 0, 0.05),
			inset 0 -0.125em 0.125em rgba(255, 255, 255, 0.5),
			0 0.25em 0.125em -0.125em rgba(0, 0, 0, 0.2),
			0 0 0.1em 0.25em inset rgba(255, 255, 255, 0.2),
			0 0 0 0 rgba(255, 255, 255, 1);
	}

	.dark-glassy-button {
		-webkit-tap-highlight-color: rgba(254, 254, 254, 0);
		background: linear-gradient(-75deg, rgba(0, 0, 0, 0.05), rgba(0, 0, 0, 0.2), rgba(0, 0, 0, 0.05));
		box-shadow:
			inset 0 0.125em 0.125em rgba(254, 254, 254, 0.05),
			inset 0 -0.125em 0.125em rgba(0, 0, 0, 0.5),
			0 0.25em 0.125em -0.125em rgba(254, 254, 254, 0.2),
			0 0 0.1em 0.25em inset rgba(0, 0, 0, 0.2),
			0 0 0 0 rgba(0, 0, 0, 1);
	}

	.dark-glassy-button:hover {
		box-shadow:
			inset 0 0.125em 0.125em rgba(254, 254, 254, 0.05),
			inset 0 -0.125em 0.125em rgba(0, 0, 0, 0.5),
			0 0.15em 0.05em -0.1em rgba(254, 254, 254, 0.25),
			0 0 0.05em 0.1em inset rgba(0, 0, 0, 0.5),
			0 0 0 0 rgba(0, 0, 0, 1);
	}

	.light-glassy-button:hover {
		box-shadow:
			inset 0 0.125em 0.125em rgba(0, 0, 0, 0.05),
			inset 0 -0.125em 0.125em rgba(255, 255, 255, 0.5),
			0 0.15em 0.05em -0.1em rgba(0, 0, 0, 0.25),
			0 0 0.05em 0.1em inset rgba(255, 255, 255, 0.5),
			0 0 0 0 rgba(255, 255, 255, 1);
	}

	.dark-glassy-button span {
		text-shadow: 0em 0.12em 0.05em rgba(254, 254, 254, 0.1);
	}

	.light-glassy-button span {
		text-shadow: 0em 0.12em 0.05em rgba(0, 0, 0, 0.1);
	}

	.dark-glassy-button:hover span {
		text-shadow: 0.025em 0.025em 0.025em rgba(254, 254, 254, 0.12);
	}

	.light-glassy-button:hover span {
		text-shadow: 0.025em 0.025em 0.025em rgba(0, 0, 0, 0.12);
	}

	.dark-glassy-button span::after {
		background: linear-gradient(
			var(--angle-2),
			rgba(0, 0, 0, 0) 0%,
			rgba(0, 0, 0, 0.5) 80% 90%,
			rgba(0, 0, 0, 0) 105%
		);
	}

	.light-glassy-button span::after {
		background: linear-gradient(
			var(--angle-2),
			rgba(255, 255, 255, 0) 0%,
			rgba(255, 255, 255, 0.5) 20% 30%,
			rgba(255, 255, 255, 0) 55%
		);
	}

	.dark-glassy-button::after {
		background: conic-gradient(
				from var(--angle-1) at 50% 50%,
				rgba(254, 254, 254, 0.5),
				rgba(254, 254, 254, 0) 5% 40%,
				rgba(254, 254, 254, 0.5) 50%,
				rgba(254, 254, 254, 0) 60% 95%,
				rgba(254, 254, 254, 0.5)
			),
			linear-gradient(180deg, rgba(0, 0, 0, 0.5), rgba(0, 0, 0, 0.5));
	}

	.light-glassy-button::after {
		background: conic-gradient(
				from var(--angle-1) at 50% 50%,
				rgba(0, 0, 0, 0.5),
				rgba(0, 0, 0, 0) 5% 40%,
				rgba(0, 0, 0, 0.5) 50%,
				rgba(0, 0, 0, 0) 60% 95%,
				rgba(0, 0, 0, 0.5)
			),
			linear-gradient(180deg, rgba(255, 255, 255, 0.5), rgba(255, 255, 255, 0.5));
	}

	.button-wrap:has(.dark-glassy-button:active) .glassy-button {
		box-shadow:
			inset 0 0.125em 0.125em rgba(254, 254, 254, 0.05),
			inset 0 -0.125em 0.125em rgba(0, 0, 0, 0.5),
			0 0.125em 0.125em -0.125em rgba(254, 254, 254, 0.2),
			0 0 0.1em 0.25em inset rgba(0, 0, 0, 0.2),
			0 0.225em 0.05em 0 rgba(254, 254, 254, 0.05),
			0 0.25em 0 0 rgba(0, 0, 0, 0.75),
			inset 0 0.25em 0.05em 0 rgba(254, 254, 254, 0.15);
	}

	.button-wrap:has(.light-glassy-button:active) .glassy-button {
		box-shadow:
			inset 0 0.125em 0.125em rgba(0, 0, 0, 0.05),
			inset 0 -0.125em 0.125em rgba(255, 255, 255, 0.5),
			0 0.125em 0.125em -0.125em rgba(0, 0, 0, 0.2),
			0 0 0.1em 0.25em inset rgba(255, 255, 255, 0.2),
			0 0.225em 0.05em 0 rgba(0, 0, 0, 0.05),
			0 0.25em 0 0 rgba(255, 255, 255, 0.75),
			inset 0 0.25em 0.05em 0 rgba(0, 0, 0, 0.15);
	}
</style>
