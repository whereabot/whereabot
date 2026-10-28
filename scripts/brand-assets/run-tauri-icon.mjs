// App icons for desktop, iOS and Android come from the Tauri CLI (app/ pins it), never from hand-sized copies.
import { execFileSync } from 'node:child_process';
import { APP_ICON, at } from './paths.mjs';

export function runTauriIcon() {
  execFileSync('npx', ['tauri', 'icon', APP_ICON, '--output', 'src-tauri/icons'], { cwd: at('app'), stdio: 'inherit' });
}
