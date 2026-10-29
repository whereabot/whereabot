#!/usr/bin/env node
// Generates every derived brand asset from the masters in brand/. Derived files are committed.
//   node scripts/generate-brand-assets.mjs           regenerate
//   node scripts/generate-brand-assets.mjs --check   regenerate, exit 1 if any committed file changed (CI)
// Needs: `npm ci` in scripts/ (svgo, sharp, font) and in app/ (Tauri CLI).
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { optimizeMasters } from './brand-assets/optimize-masters.mjs';
import { ROOT, at } from './brand-assets/paths.mjs';
import { renderSocialImage } from './brand-assets/render-social-image.mjs';
import { renderWebIcons } from './brand-assets/render-web-icons.mjs';
import { runTauriIcon } from './brand-assets/run-tauri-icon.mjs';
import { sameOutput } from './brand-assets/same-output.mjs';

// Everything this script may write. Directories are read recursively.
const OUTPUTS = ['brand', 'app/src-tauri/icons', 'app/src-tauri/gen', 'site/mark.svg', 'site/favicon.ico',
  'site/apple-touch-icon.png', 'site/icon-192.png', 'site/icon-512.png', 'site/og-image.png',
  'site/site.webmanifest'];
const SKIP = new Set(['node_modules', 'build', '.gradle']);

function snapshot() {
  const files = new Map();
  const visit = (path) => {
    if (!existsSync(path)) return;
    if (statSync(path).isDirectory()) {
      for (const name of readdirSync(path)) if (!SKIP.has(name)) visit(join(path, name));
    } else if (!path.endsWith('.md') && !path.endsWith('.icns')) { // tauri writes icon.icns with unstable entry order
      files.set(relative(ROOT, path), readFileSync(path));
    }
  };
  OUTPUTS.forEach((p) => visit(at(p)));
  return files;
}

const before = snapshot();
optimizeMasters();
await runTauriIcon();
await renderWebIcons();
await renderSocialImage();
const after = snapshot();

const names = [...new Set([...before.keys(), ...after.keys()])];
const same = await Promise.all(names.map((f) => sameOutput(f, before.get(f), after.get(f))));
const changed = names.filter((_, i) => !same[i]);
if (process.argv.includes('--check')) {
  if (changed.length) {
    console.error(`Brand assets are out of date. Run scripts/generate-brand-assets.mjs and commit:\n${changed.join('\n')}`);
    process.exit(1);
  }
  console.log('Brand assets are up to date.');
} else {
  console.log(changed.length ? `Updated ${changed.length} files.` : 'Nothing to update.');
}
