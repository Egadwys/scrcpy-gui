<script lang="ts">
  type Tone = 'none' | 'ready' | 'live' | 'warn';
  let { tone, wifi = false }: { tone: Tone; wifi?: boolean } = $props();
</script>

<div class="stage" data-tone={tone}>
  <div class="glow"></div>
  <svg viewBox="0 0 160 330" width="150" height="310" aria-hidden="true">
    <defs>
      <clipPath id="wave-clip"><rect x="30" y="206" width="100" height="40" /></clipPath>
    </defs>

    {#if wifi}
      <g class="air" class:on={tone === 'live'}>
        <path d="M52 296 a40 40 0 0 1 56 0" />
        <path d="M62 310 a26 26 0 0 1 36 0" />
        <circle cx="80" cy="324" r="3.5" />
      </g>
    {:else}
      <rect class="plug" x="70" y="266" width="20" height="26" rx="4" />
      <rect class="cable" x="78" y="292" width="4" height="38" rx="2" />
      {#if tone === 'live'}<circle class="flow" cx="80" cy="294" r="3" />{/if}
    {/if}

    <rect class="shell" x="10" y="4" width="140" height="262" rx="26" />
    <rect class="screen" x="16" y="10" width="128" height="250" rx="21" />
    <rect class="notch" x="68" y="16" width="24" height="7" rx="3.5" />

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
    position: relative;
    display: grid;
    place-items: center;
    width: 100%;
  }
  .stage[data-tone='ready'] {
    --c: #4f8dff;
  }
  .stage[data-tone='live'] {
    --c: #2ecc8f;
  }
  .stage[data-tone='warn'] {
    --c: #f5a524;
  }
  .glow {
    position: absolute;
    width: 460px;
    height: 460px;
    max-width: 100%;
    background: radial-gradient(circle, color-mix(in srgb, var(--c) 28%, transparent), transparent 62%);
    pointer-events: none;
    transition: background 0.4s;
  }
  svg {
    position: relative;
    overflow: visible;
  }
  .shell {
    fill: #0b0f0e;
    stroke: color-mix(in srgb, var(--c) 45%, #2b3036);
    stroke-width: 3;
    transition: stroke 0.4s;
  }
  .screen {
    fill: color-mix(in srgb, var(--c) 22%, #0c110f);
    transition: fill 0.4s;
  }
  .notch {
    fill: #05080a;
  }
  .disc {
    fill: var(--c);
    transition: fill 0.4s;
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
  .plug {
    fill: #3a3f46;
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
      transform: translateY(34px);
      opacity: 0;
    }
  }
  .air path {
    fill: none;
    stroke: #3a3f46;
    stroke-width: 4;
    stroke-linecap: round;
  }
  .air circle {
    fill: #3a3f46;
  }
  .air.on path,
  .air.on circle {
    stroke: var(--c);
    fill: none;
  }
  .air.on circle {
    fill: var(--c);
  }
</style>
