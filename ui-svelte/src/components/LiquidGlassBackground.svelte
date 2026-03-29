<script>
	import { onMount } from 'svelte';

	let rootElement = $state();
	// declare css variables for web-components
	onMount(() => {
		rootElement.style.setProperty('--anim--hover-time', `400ms`);
		rootElement.style.setProperty('--anim--hover-ease', `cubic-bezier(0.25, 1, 0.5, 1)`);
	});

	// update css roundess
	$effect(() => {
		if (rootElement) {
			rootElement.style.setProperty('--roundness', `12px`);
		}
	});
</script>

<div
	bind:this={rootElement}
	role="presentation"
	class="bg-wrap relative overscroll-contain">
	<div class="button-shadow dark-shadow"></div>
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

	.bg-wrap {
		position: absolute;
		overflow: hidden;
		border-radius: var(--roundness);
		background: transparent;
		pointer-events: none;
		transition: all var(--anim--hover-time) var(--anim--hover-ease);
		width: 100%;
		height: 100%;
		inset: 0;
		z-index: 0;
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

	.dark-shadow::after {
		background: linear-gradient(180deg, rgba(254, 254, 254, 0.2), rgba(254, 254, 254, 0.1));
	}
</style>
