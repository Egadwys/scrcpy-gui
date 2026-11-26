<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api, errText } from './api';
  import { store } from './store.svelte';
  import OptionsPanel from './OptionsPanel.svelte';
  import type { Settings } from './types';

  let { onclose, onsaved }: { onclose: () => void; onsaved?: () => void } = $props();

  let draft = $state<Settings>(JSON.parse(JSON.stringify(store.settings)));
  let autostart = $state(false);
  let autostartInitial = false;
  let saving = $state(false);

  const customScrcpy = $derived(!!draft.scrcpyPath.trim());
  const checking = $derived(['checking', 'downloading', 'installing'].includes(store.update?.phase ?? ''));
  const sourceLabel: Record<string, string> = {
    bundled: 'built-in',
    downloaded: 'auto-updated',
    custom: 'custom path',
    system: 'from PATH'
  };

  const scrcpyLine = $derived.by(() => {
    const t = store.tools?.scrcpy;
    if (!t) return 'scrcpy: checking…';
    if (t.error) return `scrcpy: ${t.error}`;
    const v = (t.version ?? 'scrcpy').split('<')[0].trim();
    return `${v} (${sourceLabel[t.source ?? ''] ?? 'unknown source'})`;
  });

  const updateMsg = $derived.by(() => {
    const u = store.update;
    if (!u) return '';
    switch (u.phase) {
      case 'checking':
        return 'Checking for updates…';
      case 'downloading':
        return `Downloading ${u.version} ${Math.round(u.progress ?? 0)}%`;
      case 'installing':
        return `Installing ${u.version}…`;
      case 'uptodate':
        return 'scrcpy is up to date.';
      case 'updated':
        return u.pending ? `${u.version} will be used after the current session.` : `Updated to ${u.version}.`;
      case 'error':
        return `Last check failed: ${u.message ?? 'unknown error'}`;
      default:
        return u.message ?? '';
    }
  });

  onMount(async () => {
    try {
      autostart = autostartInitial = await api.getAutostart();
    } catch {
      /* autostart unsupported on this platform: leave the toggle off */
    }
  });

  async function browse(field: 'adbPath' | 'scrcpyPath' | 'outputDir', directory = false) {
    const picked = await open({ directory, multiple: false });
    if (typeof picked === 'string') draft[field] = picked;
  }

  async function save() {
    saving = true;
    try {
      const next: Settings = JSON.parse(JSON.stringify(draft));
      next.wifiPort = Math.min(65535, Math.max(1, Math.floor(Number(next.wifiPort) || 5555)));
      next.defaults.maxSize = Math.max(0, Math.floor(Number(next.defaults.maxSize) || 0));
      next.defaults.videoBitRate = Math.max(0, Math.floor(Number(next.defaults.videoBitRate) || 0));
      next.defaults.maxFps = Math.max(0, Math.floor(Number(next.defaults.maxFps) || 0));
      await store.saveSettings(next);
      if (autostart !== autostartInitial) await api.setAutostart(autostart);
      store.toast('success', 'Preferences saved');
      onsaved?.();
      onclose();
    } catch (e) {
      store.toast('error', errText(e), 7000);
    } finally {
      saving = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={(e) => e.target === e.currentTarget && onclose()}>
  <div class="modal card" role="dialog" aria-modal="true" aria-label="Preferences">
    <header>
      <h3>Preferences</h3>
      <button class="btn ghost" aria-label="Close" onclick={onclose}>×</button>
    </header>

    <div class="body">
      <section>
        <h2>General</h2>
        <label class="field">
          Theme
          <select bind:value={draft.theme}>
            <option value="system">Follow system</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </label>
        <label class="check"><input type="checkbox" bind:checked={autostart} /> Run at startup (does not open any session)</label>
        <label class="check">
          <input type="checkbox" bind:checked={draft.keepInTray} />
          Keep running in the tray when the window is closed during a session
        </label>
      </section>

      <section>
        <h2>Tools</h2>
        <p class="muted">scrcpy and adb are built in and kept up to date. Set a path only to use your own copy.</p>
        <div class="toolline">
          <span>{scrcpyLine}</span>
          <button class="btn small" disabled={customScrcpy || checking} onclick={() => store.checkUpdate()}>
            Check now
          </button>
        </div>
        {#if updateMsg}<p class="muted">{updateMsg}</p>{/if}
        <label class="check">
          <input type="checkbox" bind:checked={draft.autoUpdateScrcpy} disabled={customScrcpy} />
          Check for a newer scrcpy every time the app starts and use it automatically
        </label>
        <label class="field">
          scrcpy path
          <div class="pick">
            <input type="text" placeholder={store.tools?.scrcpy.path ?? 'scrcpy'} bind:value={draft.scrcpyPath} />
            <button class="btn" onclick={() => browse('scrcpyPath')}>Browse</button>
          </div>
        </label>
        <label class="field">
          adb path
          <div class="pick">
            <input type="text" placeholder={store.tools?.adb.path ?? 'adb'} bind:value={draft.adbPath} />
            <button class="btn" onclick={() => browse('adbPath')}>Browse</button>
          </div>
        </label>
      </section>

      <section>
        <h2>Output</h2>
        <label class="field">
          Recordings and screenshots folder
          <div class="pick">
            <input type="text" bind:value={draft.outputDir} />
            <button class="btn" onclick={() => browse('outputDir', true)}>Browse</button>
          </div>
        </label>
      </section>

      <section>
        <h2>Default mirroring options</h2>
        <OptionsPanel bind:opts={draft.defaults} />
      </section>

      <section>
        <h2>Advanced</h2>
        <label class="field">
          TCP/IP port used by “Go wireless”
          <input type="number" min="1" max="65535" bind:value={draft.wifiPort} />
        </label>
        <label class="field">
          Extra scrcpy arguments (added to every launch)
          <input type="text" placeholder="--crop=1224:1440:0:0 --disable-screensaver" bind:value={draft.extraArgs} />
        </label>
      </section>
    </div>

    <footer>
      <button class="btn" onclick={onclose}>Cancel</button>
      <button class="btn primary" disabled={saving} onclick={save}>Save</button>
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    z-index: 50;
  }
  .modal {
    width: min(680px, 100%);
    max-height: 100%;
    display: flex;
    flex-direction: column;
  }
  .modal > header,
  .modal > footer {
    display: flex;
    align-items: center;
    padding: 12px 16px;
  }
  .modal > header {
    justify-content: space-between;
    border-bottom: 1px solid var(--border);
  }
  .modal > footer {
    justify-content: flex-end;
    gap: 8px;
    border-top: 1px solid var(--border);
  }
  .body {
    overflow: auto;
    padding: 4px 16px 16px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-top: 12px;
  }
  section p {
    margin: 0;
  }
  .toolline {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .pick {
    display: flex;
    gap: 8px;
  }
  .pick input {
    flex: 1;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }
</style>
