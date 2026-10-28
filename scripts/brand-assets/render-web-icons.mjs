// Favicons, touch icons and the web manifest for site/. Raster sizes are rendered from the app-icon master.
import { copyFileSync, writeFileSync } from 'node:fs';
import sharp from 'sharp';
import { APP_ICON, COLOURS, MARK, at } from './paths.mjs';

const SIZES = { 'apple-touch-icon.png': 180, 'icon-192.png': 192, 'icon-512.png': 512 };

const manifest = {
  name: 'Whereabot',
  short_name: 'Whereabot',
  icons: [
    { src: '/icon-192.png', sizes: '192x192', type: 'image/png', purpose: 'any maskable' },
    { src: '/icon-512.png', sizes: '512x512', type: 'image/png', purpose: 'any maskable' },
  ],
  theme_color: COLOURS.accent,
  background_color: COLOURS.background,
  display: 'browser',
};

export async function renderWebIcons() {
  for (const [name, size] of Object.entries(SIZES)) {
    await sharp(APP_ICON, { density: 384 }).resize(size, size).flatten({ background: COLOURS.background }).png().toFile(at('site', name));
  }
  copyFileSync(at('app', 'src-tauri', 'icons', 'icon.ico'), at('site', 'favicon.ico'));
  // Page markup references these two copies; they are derived, never edited.
  copyFileSync(MARK, at('site', 'mark.svg'));
  copyFileSync(MARK, at('site', 'oauth', 'consent', 'mark.svg'));
  writeFileSync(at('site', 'site.webmanifest'), JSON.stringify(manifest, null, 2) + '\n');
}
