// Tidies the master SVGs in place with svgo. Keeps viewBox and <title> (used as the accessible name).
import { readFileSync, writeFileSync } from 'node:fs';
import { optimize } from 'svgo';
import { APP_ICON, MARK } from './paths.mjs';

const config = {
  multipass: true,
  plugins: [{ name: 'preset-default', params: { overrides: { removeViewBox: false, removeTitle: false } } }],
};

export function optimizeMasters() {
  for (const file of [MARK, APP_ICON]) {
    const { data } = optimize(readFileSync(file, 'utf8'), { ...config, path: file });
    writeFileSync(file, `${data}\n`);
  }
}
