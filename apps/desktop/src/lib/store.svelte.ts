import { listen } from '@tauri-apps/api/event';
import { api, errText } from './api';
import type {
  Device,
  LaunchOptions,
  LogLine,
  SessionEnded,
  SessionInfo,
  Settings,
  Theme,
  Toast,
  ToolStatus,
  UpdateStatus
} from './types';

const MAX_LOG_LINES = 1500;

class Store {
  ready = $state(false);
  settings = $state<Settings | null>(null);
  tools = $state<ToolStatus | null>(null);
  devices = $state<Device[]>([]);
  adbError = $state<string | null>(null);
  sessions = $state<SessionInfo[]>([]);
  logs = $state<LogLine[]>([]);
  toasts = $state<Toast[]>([]);
  update = $state<UpdateStatus | null>(null);
  /** stderr lines (scrcpy warnings/errors) since the log was last opened */
  unreadErrors = $state(0);

  private toastId = 0;
  /** True while a check started from Preferences is running, so its result gets a toast. */
  private manualCheck = false;
  /** Serials the user asked to stop, so their exit is not reported as an error. */
  private stopping = new Set<string>();

  /** Is a scrcpy window currently running for this device? */
  session(serial: string): SessionInfo | undefined {
    return this.sessions.find((s) => s.serial === serial);
  }

  toast(kind: Toast['kind'], text: string, ms = 4500) {
    const id = ++this.toastId;
    this.toasts.push({ id, kind, text });
    setTimeout(() => this.dismiss(id), ms);
  }

  dismiss(id: number) {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  applyTheme(theme: Theme) {
    const root = document.documentElement;
    if (theme === 'system') root.removeAttribute('data-theme');
    else root.setAttribute('data-theme', theme);
  }

  addLog(line: LogLine) {
    if (line.stream === 'stderr') this.unreadErrors++;
    this.logs.push(line);
    if (this.logs.length > MAX_LOG_LINES) {
      this.logs.splice(0, this.logs.length - MAX_LOG_LINES);
    }
  }

  clearLogs() {
    this.logs = [];
    this.unreadErrors = 0;
  }

  markLogsRead() {
    this.unreadErrors = 0;
  }

  async refreshTools() {
    try {
      this.tools = await api.checkTools();
    } catch (e) {
      this.toast('error', errText(e));
    }
  }

  async init() {
    try {
      this.settings = await api.getSettings();
      this.applyTheme(this.settings.theme);
      const [snap, sessions] = await Promise.all([api.listDevices(), api.listSessions()]);
      this.devices = snap.devices;
      this.adbError = snap.error ?? null;
      this.sessions = sessions;
      this.update = await api.getUpdateStatus();
    } catch (e) {
      this.toast('error', errText(e));
    }
    this.ready = true;
    void this.refreshTools();

    await listen<{ devices: Device[]; error?: string | null }>('devices-changed', (ev) => {
      this.devices = ev.payload.devices;
      this.adbError = ev.payload.error ?? null;
    });
    await listen<SessionInfo[]>('sessions-changed', (ev) => {
      this.sessions = ev.payload;
    });
    await listen<UpdateStatus>('scrcpy-update', (ev) => {
      const u = ev.payload;
      this.update = u;
      const manual = this.manualCheck;
      switch (u.phase) {
        case 'updated':
          this.manualCheck = false;
          if (u.pending) {
            this.toast('success', `scrcpy ${u.version} downloaded. It will be used after the current session ends.`, 7000);
          } else {
            this.toast('success', `scrcpy updated to ${u.version}`);
            void this.refreshTools();
          }
          break;
        case 'uptodate':
          if (manual) this.toast('info', `scrcpy ${u.version ?? ''} is the latest version`);
          this.manualCheck = false;
          break;
        case 'error':
          if (manual) this.toast('error', u.message ?? 'Update check failed', 7000);
          this.manualCheck = false;
          break;
        case 'unsupported':
          this.manualCheck = false;
          break;
      }
    });
    await listen<LogLine>('log', (ev) => this.addLog(ev.payload));
    await listen<SessionEnded>('session-ended', (ev) => {
      const { serial, code } = ev.payload;
      const requested = this.stopping.delete(serial);
      if (!requested && code !== 0 && code !== null && code !== undefined) {
        this.toast('error', `scrcpy for ${serial} exited with code ${code}. Check the log for details.`, 7000);
      }
    });
  }

  async saveSettings(next: Settings) {
    await api.saveSettings(next);
    this.settings = next;
    this.applyTheme(next.theme);
    void this.refreshTools();
  }

  async checkUpdate() {
    this.manualCheck = true;
    try {
      await api.checkScrcpyUpdate();
    } catch (e) {
      this.manualCheck = false;
      this.toast('error', errText(e), 7000);
    }
  }

  async start(serial: string, options: LaunchOptions) {
    try {
      await api.startSession(serial, options);
    } catch (e) {
      this.toast('error', errText(e), 7000);
    }
  }

  async stop(serial: string) {
    this.stopping.add(serial);
    try {
      await api.stopSession(serial);
    } catch (e) {
      this.stopping.delete(serial);
      this.toast('error', errText(e));
    }
  }
}

export const store = new Store();
