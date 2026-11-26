import { invoke } from '@tauri-apps/api/core';
import type {
  DeviceSnapshot,
  LaunchOptions,
  SessionInfo,
  Settings,
  ToolStatus,
  UpdateStatus
} from './types';

/** Typed wrappers around the Rust commands in src-tauri/src/commands.rs */
export const api = {
  getSettings: () => invoke<Settings>('get_settings'),
  saveSettings: (settings: Settings) => invoke<void>('save_settings', { settings }),
  checkTools: () => invoke<ToolStatus>('check_tools'),
  listDevices: () => invoke<DeviceSnapshot>('list_devices'),
  listSessions: () => invoke<SessionInfo[]>('list_sessions'),
  startSession: (serial: string, options: LaunchOptions) =>
    invoke<void>('start_session', { serial, options }),
  stopSession: (serial: string) => invoke<void>('stop_session', { serial }),
  wifiEnable: (serial: string) => invoke<string>('wifi_enable', { serial }),
  wifiConnect: (address: string) => invoke<void>('wifi_connect', { address }),
  wifiDisconnect: (address: string) => invoke<void>('wifi_disconnect', { address }),
  wifiPair: (address: string, code: string) => invoke<void>('wifi_pair', { address, code }),
  takeScreenshot: (serial: string) => invoke<string>('take_screenshot', { serial }),
  revealPath: (path: string) => invoke<void>('reveal_path', { path }),
  getAutostart: () => invoke<boolean>('get_autostart'),
  setAutostart: (enabled: boolean) => invoke<void>('set_autostart', { enabled }),
  getUpdateStatus: () => invoke<UpdateStatus | null>('get_update_status'),
  checkScrcpyUpdate: () => invoke<void>('check_scrcpy_update')
};

/** Rust errors arrive as plain strings. */
export function errText(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error) return e.message;
  return String(e);
}

/** The UI uses number inputs that can be null/NaN when cleared. */
export function normalizeOptions(o: LaunchOptions): LaunchOptions {
  const n = (v: unknown) => Math.max(0, Math.floor(Number(v) || 0));
  return {
    ...JSON.parse(JSON.stringify(o)),
    maxSize: n(o.maxSize),
    videoBitRate: n(o.videoBitRate),
    maxFps: n(o.maxFps)
  };
}

export function cloneOptions(o: LaunchOptions): LaunchOptions {
  return JSON.parse(JSON.stringify(o));
}

export function emptyOptions(): LaunchOptions {
  return {
    maxSize: 0,
    videoBitRate: 0,
    maxFps: 0,
    videoCodec: '',
    orientation: '',
    windowTitle: '',
    turnScreenOff: false,
    stayAwake: false,
    alwaysOnTop: false,
    fullscreen: false,
    borderless: false,
    noAudio: false,
    noControl: false,
    showTouches: false,
    record: false
  };
}
