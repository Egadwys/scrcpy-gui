export interface Device {
  serial: string;
  /** device | unauthorized | offline | no-permissions | ... */
  state: string;
  model?: string | null;
  product?: string | null;
  isWifi: boolean;
}

export interface DeviceSnapshot {
  devices: Device[];
  error?: string | null;
}

export interface LaunchOptions {
  maxSize: number;
  videoBitRate: number;
  maxFps: number;
  videoCodec: '' | 'h264' | 'h265' | 'av1';
  orientation: '' | '0' | '90' | '180' | '270';
  windowTitle: string;
  turnScreenOff: boolean;
  stayAwake: boolean;
  alwaysOnTop: boolean;
  fullscreen: boolean;
  borderless: boolean;
  noAudio: boolean;
  noControl: boolean;
  showTouches: boolean;
  record: boolean;
}

export type Theme = 'system' | 'light' | 'dark';

export interface Settings {
  adbPath: string;
  scrcpyPath: string;
  outputDir: string;
  theme: Theme;
  keepInTray: boolean;
  wifiPort: number;
  autoUpdateScrcpy: boolean;
  extraArgs: string;
  defaults: LaunchOptions;
}

export interface SessionInfo {
  serial: string;
  pid: number;
  startedAt: number;
  recording?: string | null;
}

export interface SessionEnded {
  serial: string;
  code?: number | null;
  recording?: string | null;
}

export interface LogLine {
  serial: string;
  stream: 'stdout' | 'stderr';
  line: string;
}

export interface ToolInfo {
  path?: string | null;
  /** custom | downloaded | bundled | system */
  source?: string | null;
  version?: string | null;
  error?: string | null;
}

export interface ToolStatus {
  adb: ToolInfo;
  scrcpy: ToolInfo;
}

export interface Toast {
  id: number;
  kind: 'info' | 'success' | 'error';
  text: string;
}

export interface UpdateStatus {
  phase: 'checking' | 'downloading' | 'installing' | 'updated' | 'uptodate' | 'error' | 'unsupported';
  version?: string | null;
  /** 0..100 while downloading */
  progress?: number | null;
  message?: string | null;
  /** A new version is installed and will be used once running sessions end. */
  pending: boolean;
}
