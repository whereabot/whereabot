// Single place for repo paths and brand colours (colours mirror the light tokens in site/css/base.css).
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

export const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
export const at = (...parts) => join(ROOT, ...parts);

export const MARK = at('brand', 'whereabot-mark.svg');
export const APP_ICON = at('brand', 'whereabot-app-icon.svg');
export const WORDMARK = at('brand', 'whereabot-wordmark.svg');

export const COLOURS = { background: '#FBFAF7', text: '#1B1D21', muted: '#4B5058', accent: '#2F5BEA' };

export const SOCIAL = {
  title: 'Whereabot',
  tagline: ['Your agent can ask where you are.', 'Only when it asks.'],
};
