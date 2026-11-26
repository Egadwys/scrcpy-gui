#!/usr/bin/env node
// Downloads the official scrcpy release for the current platform into
// apps/desktop/src-tauri/resources/scrcpy so it gets bundled into the installer.
//
//   node scripts/fetch-scrcpy.mjs [--strict] [--force] [--platform win64|linux-x86_64|macos-aarch64|macos-x86_64]
//
// Version: scripts/scrcpy-version.txt (e.g. v4.1), or SCRCPY_VERSION=latest / SCRCPY_VERSION=v3.3.4.
// The archive is verified against the SHA256SUMS.txt published in the same release.
// Without --strict a failure only prints a warning (handy for offline `tauri dev`).

import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import {
  cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync
} from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const outDir = join(root, 'apps/desktop/src-tauri/resources/scrcpy');
const argv = process.argv.slice(2);
const strict = argv.includes('--strict');
const force = argv.includes('--force');
const platformArg = argv.includes('--platform') ? argv[argv.indexOf('--platform') + 1] : null;

const UA = { 'User-Agent': 'scrcpy-gui-fetch' };
if (process.env.GITHUB_TOKEN) UA.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;

function detectPlatform() {
  if (platformArg) return platformArg;
  const { platform, arch } = process;
  if (platform === 'win32') return 'win64';
  if (platform === 'linux' && arch === 'x64') return 'linux-x86_64';
  if (platform === 'darwin' && arch === 'arm64') return 'macos-aarch64';
  if (platform === 'darwin' && arch === 'x64') return 'macos-x86_64';
  throw new Error(`No official scrcpy build for ${platform}/${arch}`);
}

async function get(url) {
  const res = await fetch(url, { headers: UA, redirect: 'follow' });
  if (!res.ok) throw new Error(`GET ${url} -> HTTP ${res.status}`);
  return Buffer.from(await res.arrayBuffer());
}

async function resolveTag() {
  const wanted = (process.env.SCRCPY_VERSION || readFileSync(join(root, 'scripts/scrcpy-version.txt'), 'utf8')).trim();
  if (wanted !== 'latest') return wanted.startsWith('v') ? wanted : `v${wanted}`;
  const json = JSON.parse((await get('https://api.github.com/repos/Genymobile/scrcpy/releases/latest')).toString());
  return json.tag_name;
}

function expectedHash(sums, file) {
  for (const line of sums.split(/\r?\n/)) {
    const m = line.trim().match(/^([0-9a-fA-F]{64})\s+\*?(.+)$/);
    if (m && m[2] === file) return m[1].toLowerCase();
  }
  return null;
}

async function main() {
  const platform = detectPlatform();
  const isWin = platform === 'win64';
  const tag = await resolveTag();
  const exe = join(outDir, isWin ? 'scrcpy.exe' : 'scrcpy');
  const versionFile = join(outDir, 'VERSION');

  if (!force && existsSync(exe) && existsSync(versionFile) && readFileSync(versionFile, 'utf8').trim() === tag) {
    console.log(`[fetch-scrcpy] scrcpy ${tag} (${platform}) already present`);
    return;
  }

  const file = `scrcpy-${platform}-${tag}.${isWin ? 'zip' : 'tar.gz'}`;
  const base = `https://github.com/Genymobile/scrcpy/releases/download/${tag}`;
  console.log(`[fetch-scrcpy] downloading ${file}`);

  const sums = (await get(`${base}/SHA256SUMS.txt`)).toString();
  const want = expectedHash(sums, file);
  if (!want) throw new Error(`SHA256SUMS.txt of ${tag} has no entry for ${file}`);

  const data = await get(`${base}/${file}`);
  const got = createHash('sha256').update(data).digest('hex');
  if (got !== want) throw new Error(`Checksum mismatch for ${file}: expected ${want}, got ${got}`);

  const work = mkdtempSync(join(tmpdir(), 'scrcpy-'));
  try {
    const archive = join(work, file);
    const extractDir = join(work, 'x');
    mkdirSync(extractDir);
    writeFileSync(archive, data);

    // Windows: use the system bsdtar (it reads .zip); Git's GNU tar earlier on PATH cannot.
    const tarBin = process.platform === 'win32'
      ? join(process.env.SystemRoot || 'C:\\Windows', 'System32', 'tar.exe')
      : 'tar';
    execFileSync(tarBin, ['-xf', archive, '-C', extractDir], { stdio: 'inherit' });

    let src = extractDir;
    const entries = readdirSync(extractDir, { withFileTypes: true });
    if (entries.length === 1 && entries[0].isDirectory()) src = join(extractDir, entries[0].name);
    if (!existsSync(join(src, isWin ? 'scrcpy.exe' : 'scrcpy'))) throw new Error('scrcpy executable not found in archive');

    mkdirSync(outDir, { recursive: true });
    for (const name of readdirSync(outDir)) {
      if (name !== 'README.txt') rmSync(join(outDir, name), { recursive: true, force: true });
    }
    cpSync(src, outDir, { recursive: true });
    writeFileSync(versionFile, `${tag}\n`);

    // Apache-2.0 asks redistributors to ship the license text.
    try {
      writeFileSync(join(outDir, 'LICENSE.scrcpy'), await get(`https://raw.githubusercontent.com/Genymobile/scrcpy/${tag}/LICENSE`));
    } catch (e) {
      console.warn(`[fetch-scrcpy] could not fetch LICENSE: ${e.message}`);
    }
    console.log(`[fetch-scrcpy] scrcpy ${tag} ready in ${outDir}`);
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}

main().catch((e) => {
  const present = existsSync(join(outDir, 'VERSION'));
  if (strict) {
    console.error(`[fetch-scrcpy] ERROR: ${e.message}`);
    process.exit(1);
  }
  if (present) {
    console.warn(`[fetch-scrcpy] update skipped (${e.message}); keeping the existing copy`);
  } else {
    console.warn(`[fetch-scrcpy] WARNING: ${e.message}`);
    console.warn('[fetch-scrcpy] continuing without a bundled scrcpy (the app will use PATH or auto-download).');
  }
});
