<script lang="ts">
  import type { LaunchOptions } from './types';

  let { opts = $bindable() }: { opts: LaunchOptions } = $props();

  type BoolKey = {
    [K in keyof LaunchOptions]: LaunchOptions[K] extends boolean ? K : never;
  }[keyof LaunchOptions];

  const toggles: { key: BoolKey; label: string; hint: string }[] = [
    { key: 'turnScreenOff', label: 'Turn device screen off', hint: 'Mirror while the phone display is off' },
    { key: 'stayAwake', label: 'Stay awake', hint: 'Prevent sleep while plugged in' },
    { key: 'alwaysOnTop', label: 'Always on top', hint: 'Keep the mirror window above others' },
    { key: 'fullscreen', label: 'Fullscreen', hint: 'Start the mirror in fullscreen' },
    { key: 'borderless', label: 'Borderless window', hint: 'Hide the window decorations' },
    { key: 'noAudio', label: 'No audio', hint: 'Do not forward device audio' },
    { key: 'noControl', label: 'View only', hint: 'Disable mouse and keyboard control' },
    { key: 'showTouches', label: 'Show touches', hint: 'Display touch points on the device' },
    { key: 'record', label: 'Record to file', hint: 'Save a .mkv into the output folder' }
  ];
</script>

<div class="grid">
  <label class="field">
    Max size (px)
    <input type="number" min="0" step="1" list="sizes" placeholder="Auto" bind:value={opts.maxSize} />
    <datalist id="sizes">
      <option value="480"></option>
      <option value="720"></option>
      <option value="1024"></option>
      <option value="1280"></option>
      <option value="1920"></option>
    </datalist>
  </label>

  <label class="field">
    Bit rate (Mbps)
    <input type="number" min="0" step="1" placeholder="8 (default)" bind:value={opts.videoBitRate} />
  </label>

  <label class="field">
    Max FPS
    <input type="number" min="0" step="1" placeholder="Auto" bind:value={opts.maxFps} />
  </label>

  <label class="field">
    Video codec
    <select bind:value={opts.videoCodec}>
      <option value="">Default (H.264)</option>
      <option value="h264">H.264</option>
      <option value="h265">H.265</option>
      <option value="av1">AV1</option>
    </select>
  </label>

  <label class="field">
    Orientation
    <select bind:value={opts.orientation}>
      <option value="">Auto</option>
      <option value="0">0°</option>
      <option value="90">90°</option>
      <option value="180">180°</option>
      <option value="270">270°</option>
    </select>
  </label>

  <label class="field">
    Window title
    <input type="text" placeholder="Device name" bind:value={opts.windowTitle} />
  </label>
</div>

<div class="toggles">
  {#each toggles as t (t.key)}
    <label title={t.hint}>
      <input type="checkbox" bind:checked={opts[t.key]} />
      {t.label}
    </label>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 10px;
  }
  .toggles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 6px 14px;
    margin-top: 12px;
  }
  .toggles label {
    display: flex;
    align-items: center;
    gap: 7px;
    cursor: pointer;
  }
</style>
