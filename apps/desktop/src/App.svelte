<script lang="ts">
  import { onMount } from 'svelte';
  import { api, cloneOptions, emptyOptions, errText, normalizeOptions } from './lib/api';
  import { store } from './lib/store.svelte';
  import LogPanel from './lib/LogPanel.svelte';
  import Modal from './lib/Modal.svelte';
  import OptionsPanel from './lib/OptionsPanel.svelte';
  import PhoneStage from './lib/PhoneStage.svelte';
  import Preferences from './lib/Preferences.svelte';
  import Toasts from './lib/Toasts.svelte';
  import WifiPanel from './lib/WifiPanel.svelte';
  import type { LaunchOptions } from './lib/types';

  type Tone = 'none' | 'ready' | 'live' | 'warn';
  type Action = 'start' | 'stop' | 'prefs' | null;

  let panel = $state<'' | 'prefs' | 'log' | 'options' | 'wifi'>('');
  let pick = $state<string | null>(null);
  let launchOpts = $state<LaunchOptions>(emptyOptions());
  let now = $state(Date.now());
  let busy = $state(false);

  onMount(() => {
    void store.init().then(() => {
      if (store.settings) launchOpts = cloneOptions(store.settings.defaults);
    });
    const t = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(t);
  });

  // Prefer the user's pick, then a device that is mirroring, then the first one.
  const device = $derived(
    store.devices.find((d) => d.serial === pick) ??
      store.devices.find((d) => store.session(d.serial)) ??
      store.devices[0]
  );
  const session = $derived(device ? store.session(device.serial) : undefined);
  const name = $derived(device ? (device.model ?? device.product ?? device.serial) : '');

  function clock(totalSeconds: number) {
    const s = Math.max(0, Math.floor(totalSeconds));
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = String(s % 60).padStart(2, '0');
    return h > 0 ? `${h}:${String(m).padStart(2, '0')}:${sec}` : `${String(m).padStart(2, '0')}:${sec}`;
  }

  const view = $derived.by((): { tone: Tone; kicker: string; title: string; sub: string; action: Action; label: string } => {
    const scrcpyErr = store.tools?.scrcpy.error;
    if (scrcpyErr) {
      return { tone: 'warn', kicker: 'Setup needed', title: 'scrcpy not found', sub: scrcpyErr, action: 'prefs', label: 'Open Preferences' };
    }
    if (store.adbError) {
      return { tone: 'warn', kicker: 'Setup needed', title: 'adb problem', sub: store.adbError, action: 'prefs', label: 'Open Preferences' };
    }
    if (!device) {
      return {
        tone: 'none',
        kicker: 'Waiting for a device',
        title: 'No device',
        sub: 'Plug in your phone with USB debugging enabled, or connect over Wi-Fi.',
        action: null,
        label: ''
      };
    }
    if (device.state !== 'device') {
      const map: Record<string, [string, string]> = {
        unauthorized: ['Unauthorized', 'Unlock the phone and accept “Allow USB debugging?”.'],
        offline: ['Offline', 'Replug the cable, or run “adb kill-server” in a terminal.'],
        'no-permissions': ['No permission', 'Linux needs udev rules for this device. See docs/TROUBLESHOOTING.md.']
      };
      const [title, sub] = map[device.state] ?? [device.state, `${name} reported state “${device.state}”.`];
      return { tone: 'warn', kicker: name, title, sub, action: null, label: '' };
    }
    if (session) {
      const rec = session.recording ? ' Recording to file.' : '';
      return {
        tone: 'live',
        kicker: `Mirroring for ${clock(now / 1000 - session.startedAt)}`,
        title: 'Mirroring',
        sub: `${name} is mirrored on this computer.${rec}`,
        action: 'stop',
        label: 'Stop'
      };
    }
    return {
      tone: 'ready',
      kicker: device.isWifi ? 'Connected over Wi-Fi' : 'Connected over USB',
      title: 'Ready',
      sub: `${name} is ready to mirror.`,
      action: 'start',
      label: 'Start mirroring'
    };
  });

  function act() {
    if (!device) return;
    if (view.action === 'start') void store.start(device.serial, normalizeOptions(launchOpts));
    else if (view.action === 'stop') void store.stop(device.serial);
    else if (view.action === 'prefs') panel = 'prefs';
  }

  async function screenshot() {
    if (!device) return;
    busy = true;
    try {
      const path = await api.takeScreenshot(device.serial);
      store.toast('success', `Screenshot saved: ${path}`);
    } catch (e) {
      store.toast('error', errText(e), 7000);
    } finally {
      busy = false;
    }
  }

  const updateNote = $derived.by(() => {
    const u = store.update;
    if (!u) return '';
    if (u.phase === 'downloading') {
      const pct = u.progress != null ? ` ${Math.round(u.progress)}%` : '';
      return `Updating scrcpy to ${u.version}…${pct}`;
    }
    if (u.phase === 'installing') return `Installing scrcpy ${u.version}…`;
    if (u.phase === 'updated' && u.pending) return `scrcpy ${u.version} will be used after this session`;
    return '';
  });

  const canUse = $derived(!!device && device.state === 'device');
  const logDot = $derived(store.unreadErrors > 0 ? 'bad' : store.sessions.length > 0 ? 'ok' : '');
</script>

<div class="app">
  <nav>
    <button class="link" onclick={() => (panel = 'log')}>
      Log {#if logDot}<span class="dot {logDot}"></span>{/if}
    </button>
    <button class="link" onclick={() => (panel = 'prefs')}>Preferences</button>
  </nav>

  <main>
    {#if !store.ready}
      <p class="muted">Loading…</p>
    {:else}
      {#if store.devices.length > 1}
        <select class="picker" aria-label="Device" value={device?.serial} onchange={(e) => (pick = e.currentTarget.value)}>
          {#each store.devices as d (d.serial)}
            <option value={d.serial}>{d.model ?? d.product ?? d.serial}{d.isWifi ? ' (Wi-Fi)' : ' (USB)'}</option>
          {/each}
        </select>
      {/if}

      <PhoneStage tone={view.tone} wifi={!!device?.isWifi} connected={!!device} />

      <div class="text">
        <div class="kicker">{view.kicker}</div>
        <h1>{view.title}</h1>
        <p class="sub">{view.sub}</p>
      </div>

      {#if view.action}
        <button class="pill {view.action}" onclick={act}>{view.label}</button>
      {/if}

      <div class="more">
        <button class="link small" disabled={!canUse} onclick={() => (panel = 'options')}>Options</button>
        <button class="link small" disabled={!canUse || busy} onclick={screenshot}>Screenshot</button>
        <button class="link small" onclick={() => (panel = 'wifi')}>Wireless</button>
      </div>
    {/if}
  </main>
</div>

{#if updateNote}
  <div class="upd" role="status">{updateNote}</div>
{/if}

{#if panel === 'prefs'}
  <Preferences
    onclose={() => (panel = '')}
    onsaved={() => store.settings && (launchOpts = cloneOptions(store.settings.defaults))}
  />
{:else if panel === 'log'}
  <LogPanel onclose={() => (panel = '')} />
{:else if panel === 'options'}
  <Modal title="Mirroring options" onclose={() => (panel = '')}>
    <OptionsPanel bind:opts={launchOpts} />
    <div class="foot">
      <button class="btn small" onclick={() => store.settings && (launchOpts = cloneOptions(store.settings.defaults))}>
        Reset to defaults
      </button>
      <span class="muted">{session ? 'Applies the next time you start.' : 'Used for the next session.'}</span>
    </div>
  </Modal>
{:else if panel === 'wifi'}
  <Modal title="Wireless" onclose={() => (panel = '')}>
    <WifiPanel {device} />
  </Modal>
{/if}
<Toasts />

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  nav {
    display: flex;
    justify-content: flex-end;
    gap: 22px;
    padding: 14px 24px 0;
  }
  .link {
    appearance: none;
    border: 0;
    background: transparent;
    color: var(--muted);
    font: inherit;
    font-size: 15px;
    cursor: pointer;
    padding: 2px 0;
  }
  .link:hover:not(:disabled) {
    color: var(--text);
  }
  .link:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .link.small {
    font-size: 13px;
  }
  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-left: 4px;
    background: var(--muted);
  }
  .dot.bad {
    background: var(--danger);
  }
  .dot.ok {
    background: var(--success);
  }
  main {
    flex: 1;
    min-height: 0;
    overflow: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    padding: 8px 24px 24px;
    text-align: center;
  }
  .picker {
    max-width: 280px;
  }
  .text {
    max-width: 420px;
  }
  .kicker {
    font-weight: 600;
    color: var(--muted);
    font-size: 15px;
    font-variant-numeric: tabular-nums;
  }
  h1 {
    font-size: 36px;
    margin: 4px 0 6px;
    letter-spacing: -0.01em;
  }
  .sub {
    margin: 0;
    color: var(--muted);
    font-size: 15px;
    word-break: break-word;
  }
  .pill {
    appearance: none;
    border: 0;
    border-radius: 999px;
    padding: 14px 34px;
    min-width: 160px;
    font: inherit;
    font-weight: 700;
    font-size: 16px;
    cursor: pointer;
    margin-top: 6px;
    transition: filter 0.15s, transform 0.05s;
  }
  .pill:hover {
    filter: brightness(1.15);
  }
  .pill:active {
    transform: translateY(1px);
  }
  .pill.start {
    background: color-mix(in srgb, var(--accent) 26%, var(--surface));
    color: var(--accent);
  }
  .pill.stop {
    background: color-mix(in srgb, var(--danger) 22%, var(--surface));
    color: var(--danger);
  }
  .pill.prefs {
    background: var(--surface-2);
    color: var(--text);
  }
  .more {
    display: flex;
    gap: 20px;
    margin-top: 4px;
  }
  .upd {
    position: fixed;
    left: 50%;
    bottom: 12px;
    transform: translateX(-50%);
    padding: 5px 12px;
    border-radius: 999px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--muted);
    font-size: 12px;
    box-shadow: var(--shadow);
    white-space: nowrap;
    z-index: 30;
  }
  .foot {
    display: flex;
    align-items: center;
    gap: 10px;
  }
</style>
