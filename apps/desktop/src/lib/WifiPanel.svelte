<script lang="ts">
  import { api, errText } from './api';
  import { store } from './store.svelte';
  import type { Device } from './types';

  let { device }: { device?: Device } = $props();

  let address = $state('');
  let pairAddress = $state('');
  let pairCode = $state('');
  let busy = $state(false);

  const canGoWireless = $derived(!!device && !device.isWifi && device.state === 'device');
  const isWifiDevice = $derived(!!device && device.isWifi);

  async function run(fn: () => Promise<string | void>) {
    busy = true;
    try {
      const msg = await fn();
      if (msg) store.toast('success', msg, 7000);
    } catch (e) {
      store.toast('error', errText(e), 7000);
    } finally {
      busy = false;
    }
  }

  const withPort = (a: string) => {
    const t = a.trim();
    return t.includes(':') ? t : `${t}:${store.settings?.wifiPort ?? 5555}`;
  };

  const goWireless = () =>
    run(async () => `Wireless ready at ${await api.wifiEnable(device!.serial)}. You can unplug the cable.`);
  const disconnect = () =>
    run(async () => {
      await api.wifiDisconnect(device!.serial);
      return `Disconnected ${device!.serial}`;
    });
  const connect = () =>
    run(async () => {
      const a = withPort(address);
      await api.wifiConnect(a);
      return `Connected to ${a}`;
    });
  const pair = () =>
    run(async () => {
      await api.wifiPair(pairAddress.trim(), pairCode.trim());
      return 'Paired. Now connect using the IP and port from the main Wireless debugging screen.';
    });
</script>

{#if canGoWireless}
  <div class="block">
    <p><strong>{device?.model ?? device?.serial}</strong> is on USB.</p>
    <p class="muted">Switches it to TCP/IP over your Wi-Fi so you can unplug the cable. The phone must be on the same network.</p>
    <button class="btn primary" disabled={busy} onclick={goWireless}>Go wireless</button>
  </div>
{:else if isWifiDevice}
  <div class="block">
    <p><strong>{device?.model ?? device?.serial}</strong> is connected over Wi-Fi.</p>
    <button class="btn" disabled={busy} onclick={disconnect}>Disconnect</button>
  </div>
{/if}

<div class="block">
  <strong>Connect by address</strong>
  <div class="line">
    <input
      type="text"
      placeholder="192.168.1.23 or 192.168.1.23:5555"
      bind:value={address}
      onkeydown={(e) => e.key === 'Enter' && address.trim() && !busy && connect()}
    />
    <button class="btn primary" disabled={busy || !address.trim()} onclick={connect}>Connect</button>
  </div>
</div>

<details>
  <summary>Pair a new device (Android 11+ Wireless debugging)</summary>
  <p class="muted">
    On the phone: Developer options → Wireless debugging → <em>Pair device with pairing code</em>. Enter the pairing
    address and the 6-digit code here.
  </p>
  <div class="line">
    <input type="text" placeholder="192.168.1.23:37115" bind:value={pairAddress} />
    <input class="code" type="text" inputmode="numeric" placeholder="123456" bind:value={pairCode} />
    <button class="btn" disabled={busy || !pairAddress.trim() || !pairCode.trim()} onclick={pair}>Pair</button>
  </div>
</details>

<style>
  .block {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
  }
  p {
    margin: 0;
  }
  .line {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    width: 100%;
  }
  .line input {
    flex: 1;
    min-width: 180px;
  }
  .line input.code {
    flex: 0 0 100px;
    min-width: 0;
  }
  details {
    border-top: 1px solid var(--border);
    padding-top: 10px;
  }
  summary {
    cursor: pointer;
    margin-bottom: 8px;
  }
  details p {
    margin-bottom: 8px;
  }
</style>
