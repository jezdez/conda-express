/**
 * Write conda configuration files to the bootstrapped prefix so that
 * `conda install` works at runtime via emscripten-forge.
 */

import type { EmscriptenFS } from './memfs';

export interface CondaConfigOptions {
  prefix: string;
  fs: EmscriptenFS;
  /** Default channel for runtime installs (default: emscripten-forge) */
  channel?: string;
}

/**
 * Write a `.condarc` to the prefix that configures conda for
 * emscripten-forge runtime installs.
 */
export function writeCondaConfig(options: CondaConfigOptions): void {
  const { prefix, fs, channel = 'https://repo.mamba.pm/emscripten-forge' } = options;
  const normalizedPrefix = prefix.endsWith('/') ? prefix.slice(0, -1) : prefix;

  const condarc = [
    'channels:',
    `  - ${channel}`,
    '  - nodefaults',
    '',
    `root_prefix: ${normalizedPrefix}`,
    `target_prefix_override: ${normalizedPrefix}`,
    '',
    'auto_activate_base: true',
    'always_yes: true',
    '',
  ].join('\n');

  ensureDir(fs, `${normalizedPrefix}`);
  fs.writeFile(`${normalizedPrefix}/.condarc`, condarc);
}

/**
 * Write a minimal `.cx.json` marker so we know this prefix was bootstrapped
 * by cx-web.
 */
export function writeCxMarker(
  options: CondaConfigOptions & {
    totalPackages: number;
    totalFiles: number;
    totalSize: number;
  }
): void {
  const { prefix, fs, totalPackages, totalFiles, totalSize } = options;
  const normalizedPrefix = prefix.endsWith('/') ? prefix.slice(0, -1) : prefix;

  const marker = JSON.stringify(
    {
      bootstrapped_by: 'cx-web',
      timestamp: new Date().toISOString(),
      total_packages: totalPackages,
      total_files: totalFiles,
      total_size: totalSize,
    },
    null,
    2
  );

  fs.writeFile(`${normalizedPrefix}/.cx.json`, marker);
}

function ensureDir(fs: EmscriptenFS, path: string): void {
  if (!fs.analyzePath(path).exists) {
    const parts = path.split('/').filter(Boolean);
    let current = '';
    for (const part of parts) {
      current += '/' + part;
      if (!fs.analyzePath(current).exists) {
        fs.mkdir(current);
      }
    }
  }
}
