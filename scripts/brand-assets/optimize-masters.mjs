// Tidies the app-icon master in place with svgo, then derives the mark from it.
// Keeps viewBox and <title> (used as the accessible name).
import { readFileSync, writeFileSync } from 'node:fs';
import { optimize } from 'svgo';
import { APP_ICON, MARK } from './paths.mjs';

const config = {
  multipass: true,
  plugins: [{ name: 'preset-default', params: { overrides: { removeViewBox: false, removeTitle: false } } }],
};
// The robot pin, cropped from the 1024 icon: the full scene is unreadable at 16 px, the crop is not.
const MARK_VIEWBOX = '232 60 560 560';

export function optimizeMasters() {
  const { data } = optimize(readFileSync(APP_ICON, 'utf8'), { ...config, path: APP_ICON });
  writeFileSync(APP_ICON, `${data}\n`);
  const root = `<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="${MARK_VIEWBOX}">`;
  writeFileSync(MARK, `${data.replace(/^<svg[^>]*>/, root)}\n`);
}
