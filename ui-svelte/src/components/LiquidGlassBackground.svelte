<script>
	import { onMount } from 'svelte';

	let rootElement = $state();

	onMount(() => {
		rootElement.style.setProperty('--anim--hover-time', `400ms`);
		rootElement.style.setProperty('--anim--hover-ease', `cubic-bezier(0.25, 1, 0.5, 1)`);
		rootElement.style.setProperty('--roundness', `12px`);
	});
</script>

<div
	bind:this={rootElement}
	role="presentation"
	class="bg-wrap">
	<div class="glass-bg"></div>
	<div class="glass-border"></div>
	<div class="glass-filter"></div>
</div>

<svg style="display: none">
	<filter id="lg-bg-dist" x="0%" y="0%" width="100%" height="100%">
		<feTurbulence type="fractalNoise" baseFrequency="0.012 0.012" numOctaves="3" seed="42" result="noise" />
		<feGaussianBlur in="noise" stdDeviation="3" result="blurred" />
		<feDisplacementMap in="SourceGraphic" in2="blurred" scale="80" xChannelSelector="R" yChannelSelector="G" />
	</filter>
</svg>

<style>
	.bg-wrap {
		position: absolute;
		overflow: hidden;
		border-radius: var(--roundness, 12px);
		pointer-events: none;
		width: 100%;
		height: 100%;
		inset: 0;
		z-index: 0;
	}

	.glass-bg {
		position: absolute;
		inset: 0;
		border-radius: inherit;
		background: linear-gradient(
			-75deg,
			rgba(0, 0, 0, 0.45),
			rgba(0, 0, 0, 0.75),
			rgba(0, 0, 0, 0.45)
		);
		box-shadow:
			inset 0 0.125em 0.125em rgba(254, 254, 254, 0.05),
			inset 0 -0.125em 0.125em rgba(0, 0, 0, 0.5),
			0 0.25em 0.125em -0.125em rgba(254, 254, 254, 0.2),
			0 0 0.1em 0.25em inset rgba(0, 0, 0, 0.2);
	}

	.glass-border {
		--border-width: 1px;
		position: absolute;
		inset: 0;
		border-radius: inherit;
		padding: var(--border-width);
		background: conic-gradient(
				from -75deg at 50% 50%,
				rgba(254, 254, 254, 0.35),
				rgba(254, 254, 254, 0) 5% 40%,
				rgba(254, 254, 254, 0.35) 50%,
				rgba(254, 254, 254, 0) 60% 95%,
				rgba(254, 254, 254, 0.35)
			),
			linear-gradient(180deg, rgba(0, 0, 0, 0.4), rgba(0, 0, 0, 0.4));
		mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0);
		mask-composite: exclude;
		z-index: 1;
	}

	.glass-filter {
		position: absolute;
		inset: 0;
		border-radius: inherit;
		z-index: 0;
		-webkit-backdrop-filter: blur(16px);
		backdrop-filter: blur(16px);
		filter: url(#lg-bg-dist) saturate(130%);
		isolation: isolate;
	}
</style>
