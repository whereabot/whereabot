// App icons for desktop, iOS and Android come from the Tauri CLI (app/ pins it), never from hand-sized copies.
import { execFileSync } from 'node:child_process';
import { readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import sharp from 'sharp';
import { APP_ICON, at } from './paths.mjs';

const IOS_ICONS = at('app', 'src-tauri', 'gen', 'apple', 'Assets.xcassets', 'AppIcon.appiconset');

export async function runTauriIcon() {
  execFileSync('npx', ['tauri', 'icon', APP_ICON, '--output', 'src-tauri/icons'], { cwd: at('app'), stdio: 'inherit' });
  // Tauri writes RGBA even when every pixel is opaque; App Store rejects any alpha channel in iOS icons.
  for (const name of readdirSync(IOS_ICONS).filter((n) => n.endsWith('.png'))) {
    const file = join(IOS_ICONS, name);
    writeFileSync(file, await sharp(file).removeAlpha().png().toBuffer());
  }
}
