<script lang="ts">
  import { untrack } from 'svelte';

  type Tone = 'none' | 'ready' | 'live' | 'warn';
  let { tone, wifi = false, connected = false }: { tone: Tone; wifi?: boolean; connected?: boolean } = $props();

  // none = disconnected (red), ready / live = connected (green), warn = needs attention (amber)
  const color = $derived(tone === 'none' ? 'red' : tone === 'warn' ? 'amber' : 'green');
  const speed = $derived(tone === 'live' ? 'fast' : tone === 'none' ? 'slow' : 'mid');

  // Remember how the last device was attached, so the matching animation plays when it goes away.
  let wireless = $state(wifi);
  $effect(() => {
    if (connected) wireless = wifi;
  });
  const isWireless = $derived(connected ? wifi : wireless);

  // One ripple at the port every time a device is plugged in or removed (not on first render).
  let pulse = $state(0);
  let prev: boolean | undefined;
  $effect(() => {
    const now = connected;
    if (prev !== undefined && prev !== now) pulse = untrack(() => pulse) + 1;
    prev = now;
  });
</script>

<div class="stage" data-color={color} data-speed={speed}>
  <div class="glow green" class:on={color === 'green'}><div class="core"></div></div>
  <div class="glow red" class:on={color === 'red'}><div class="core"></div></div>
  <div class="glow amber" class:on={color === 'amber'}><div class="core"></div></div>

  <svg viewBox="0 0 160 352" width="150" height="330" aria-hidden="true">
    <defs>
      <clipPath id="wave-clip"><rect x="30" y="206" width="100" height="40" /></clipPath>
    </defs>

    {#if isWireless}
      <g class="air" class:on={connected}>
        <path style="--i: 2" d="M52 296 a40 40 0 0 1 56 0" />
        <path style="--i: 1" d="M62 310 a26 26 0 0 1 36 0" />
        <circle style="--i: 0" cx="80" cy="324" r="3.5" />
      </g>
    {:else}
      <!-- Slides up into the phone when a device appears, drops away when it disconnects. -->
      <g class="plug-group" class:out={!connected}>
        <rect class="plug" x="70" y="262" width="20" height="28" rx="4" />
        <rect class="contacts" x="75" y="278" width="10" height="3" rx="1.5" />
        <rect class="cable" x="78" y="290" width="4" height="32" rx="2" />
        {#if tone === 'live'}<circle class="flow" cx="80" cy="292" r="3" />{/if}
      </g>
    {/if}

    <rect class="shell" x="10" y="4" width="140" height="262" rx="26" />
    <rect class="screen" x="16" y="10" width="128" height="250" rx="21" />
    <rect class="notch" x="68" y="16" width="24" height="7" rx="3.5" />

    {#if pulse > 0}
      {#key pulse}
        <circle class="ripple" cx="80" cy={isWireless ? 318 : 268} r="9" />
      {/key}
    {/if}

    <circle class="halo" cx="80" cy="104" r="26" />
    <circle class="disc" cx="80" cy="104" r="26" />
    {#if tone === 'live'}
      <path class="ico" d="M68 104 l8 8 l16 -17" />
    {:else if tone === 'ready'}
      <path class="ico solid" d="M75 93 L94 104 L75 115 Z" />
    {:else if tone === 'warn'}
      <path class="ico" d="M80 91 v16 M80 116 v1" />
    {:else}
      <g class="ico dots"><circle cx="68" cy="104" r="2.5" /><circle cx="80" cy="104" r="2.5" /><circle cx="92" cy="104" r="2.5" /></g>
    {/if}

    <rect class="panel" x="24" y="186" width="112" height="62" rx="10" />
    <text class="cap" x="34" y="204">Scrcpy</text>
    <text class="cap live" x="126" y="204" text-anchor="end">{tone === 'live' ? '● Live' : ''}</text>
    <g clip-path="url(#wave-clip)">
      <path
        class="wave"
        class:on={tone === 'live'}
        d="M22 228 q8 -16 16 0 t16 0 t16 0 t16 0 t16 0 t16 0 t16 0 t16 0"
      />
    </g>
  </svg>
</div>

<style>
  .stage {
    --c: #7b8794;
    --dur: 3.2s;
    position: relative;
    display: grid;
    place-items: center;
    width: 100%;
  }
  .stage[data-color='green'] {
    --c: #2ecc8f;
  }
  .stage[data-color='red'] {
    --c: #f2545b;
  }
  .stage[data-color='amber'] {
    --c: #f5a524;
  }
  .stage[data-speed='fast'] {
    --dur: 2.2s;
  }
  .stage[data-speed='slow'] {
    --dur: 4.2s;
  }

  /* ---- background glow: three layers cross-fade, the visible one breathes ---- */
  .glow {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 460px;
    height: 460px;
    max-width: 100%;
    transform: translate(-50%, -50%);
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.7s ease;
  }
  .glow.on {
    opacity: 1;
  }
  .core {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    animation: breathe var(--dur) ease-in-out infinite;
  }
  .glow.green .core {
    background: radial-gradient(circle, rgba(46, 204, 143, 0.32), transparent 62%);
  }
  .glow.red .core {
    background: radial-gradient(circle, rgba(242, 84, 91, 0.3), transparent 62%);
  }
  .glow.amber .core {
    background: radial-gradient(circle, rgba(245, 165, 36, 0.28), transparent 62%);
  }
  @keyframes breathe {
    0%,
    100% {
      transform: scale(0.82);
      opacity: 0.6;
    }
    50% {
      transform: scale(1.06);
      opacity: 1;
    }
  }

  svg {
    position: relative;
    overflow: visible;
  }

  /* ---- phone ---- */
  .shell {
    fill: #0b0f0e;
    stroke: color-mix(in srgb, var(--c) 45%, #2b3036);
    stroke-width: 3;
    transition: stroke 0.5s;
  }
  .screen {
    fill: color-mix(in srgb, var(--c) 20%, #0c110f);
    transition: fill 0.5s;
  }
  .notch {
    fill: #05080a;
  }

  /* ---- the breathing circle ---- */
  .disc,
  .halo {
    fill: var(--c);
    transition: fill 0.5s;
    transform-box: fill-box;
    transform-origin: center;
  }
  .disc {
    animation: disc var(--dur) ease-in-out infinite;
  }
  .halo {
    animation: halo var(--dur) ease-out infinite;
  }
  @keyframes disc {
    0%,
    100% {
      transform: scale(0.94);
    }
    50% {
      transform: scale(1.08);
    }
  }
  @keyframes halo {
    0% {
      transform: scale(1);
      opacity: 0.45;
    }
    100% {
      transform: scale(1.9);
      opacity: 0;
    }
  }
  .ico {
    fill: none;
    stroke: #fff;
    stroke-width: 5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .ico.solid,
  .ico.dots {
    fill: #fff;
    stroke: none;
  }

  .panel {
    fill: rgba(255, 255, 255, 0.1);
  }
  .cap {
    fill: #fff;
    font: 600 10px system-ui, sans-serif;
  }
  .cap.live {
    fill: var(--c);
  }
  .wave {
    fill: none;
    stroke: var(--c);
    stroke-width: 2.5;
    stroke-linecap: round;
    opacity: 0.35;
  }
  .wave.on {
    opacity: 1;
    animation: slide 1.6s linear infinite;
  }
  @keyframes slide {
    to {
      transform: translateX(-32px);
    }
  }

  /* ---- USB plug: snaps in with a little overshoot, drops away smoothly ---- */
  .plug-group {
    transition: transform 0.65s cubic-bezier(0.34, 1.7, 0.5, 1);
  }
  .plug-group.out {
    transform: translateY(24px);
    transition: transform 0.5s cubic-bezier(0.5, 0, 0.8, 0.4);
  }
  .plug {
    fill: #3a3f46;
  }
  .contacts {
    fill: #6b7480;
  }
  .cable {
    fill: #2a2f35;
  }
  .flow {
    fill: var(--c);
    animation: flow 1.6s ease-in infinite;
  }
  @keyframes flow {
    from {
      transform: translateY(0);
      opacity: 1;
    }
    to {
      transform: translateY(28px);
      opacity: 0;
    }
  }

  /* ripple at the port on plug / unplug */
  .ripple {
    fill: none;
    stroke: var(--c);
    stroke-width: 2.5;
    transform-box: fill-box;
    transform-origin: center;
    animation: ripple 0.9s ease-out forwards;
  }
  @keyframes ripple {
    from {
      transform: scale(0.5);
      opacity: 0.9;
    }
    to {
      transform: scale(4.5);
      opacity: 0;
    }
  }

  /* ---- Wi-Fi variant ---- */
  .air path {
    fill: none;
    stroke: #3a3f46;
    stroke-width: 4;
    stroke-linecap: round;
    transition: stroke 0.5s;
  }
  .air circle {
    fill: #3a3f46;
    transition: fill 0.5s;
  }
  .air.on path {
    stroke: var(--c);
    animation: arc 1.8s ease-in-out infinite;
    animation-delay: calc(var(--i) * 0.2s);
  }
  .air.on circle {
    fill: var(--c);
    animation: arc 1.8s ease-in-out infinite;
  }
  @keyframes arc {
    0%,
    100% {
      opacity: 0.4;
    }
    40% {
      opacity: 1;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .core,
    .disc,
    .halo,
    .wave,
    .flow,
    .ripple,
    .air path,
    .air circle {
      animation: none;
    }
    .plug-group,
    .plug-group.out {
      transition: none;
    }
  }
</style>
