<script lang="ts">
  import { tick } from 'svelte';
  import { store } from './store.svelte';

  let { onclose }: { onclose: () => void } = $props();

  let filter = $state('');
  let box = $state<HTMLDivElement>();
  let stick = true;

  const serials = $derived([...new Set(store.logs.map((l) => l.serial))]);
  const lines = $derived(filter ? store.logs.filter((l) => l.serial === filter) : store.logs);

  function onScroll() {
    if (!box) return;
    stick = box.scrollHeight - box.scrollTop - box.clientHeight < 24;
  }

  $effect(() => {
    void lines.length;
    store.markLogsRead();
    if (stick) {
      tick().then(() => {
        if (box) box.scrollTop = box.scrollHeight;
      });
    }
  });

  async function copy() {
    const text = lines.map((l) => `[${l.serial}] ${l.line}`).join('\n');
    try {
      await navigator.clipboard.writeText(text);
      store.toast('info', 'Log copied');
    } catch {
      store.toast('error', 'Could not copy the log');
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={(e) => e.target === e.currentTarget && onclose()}>
  <section class="sheet" aria-label="Log">
    <header>
      <h3>Log</h3>
      <div class="tools">
        {#if serials.length > 1}
          <select bind:value={filter} aria-label="Filter log by device">
            <option value="">All devices</option>
            {#each serials as s (s)}<option value={s}>{s}</option>{/each}
          </select>
        {/if}
        <button class="btn small" onclick={copy} disabled={!lines.length}>Copy</button>
        <button class="btn small" onclick={() => store.clearLogs()} disabled={!store.logs.length}>Clear</button>
        <button class="btn small ghost" aria-label="Close" onclick={onclose}>×</button>
      </div>
    </header>
    <div class="log mono" bind:this={box} onscroll={onScroll}>
      {#each lines as l, i (i)}
        <div class:err={l.stream === 'stderr'}>
          {#if serials.length > 1 && !filter}<span class="tag">[{l.serial}]</span>{/if}{l.line}
        </div>
      {:else}
        <div class="muted">scrcpy output appears here once you start mirroring.</div>
      {/each}
    </div>
  </section>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: flex-end;
    z-index: 40;
  }
  .sheet {
    width: 100%;
    height: 58%;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-top: 1px solid var(--border);
    border-radius: 14px 14px 0 0;
    box-shadow: var(--shadow);
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 14px;
    border-bottom: 1px solid var(--border);
  }
  .tools {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .log {
    flex: 1;
    overflow: auto;
    padding: 8px 14px;
    background: var(--surface-2);
    white-space: pre-wrap;
    word-break: break-all;
    user-select: text;
  }
  .err {
    color: var(--warn);
  }
  .tag {
    color: var(--muted);
    margin-right: 6px;
  }
</style>
