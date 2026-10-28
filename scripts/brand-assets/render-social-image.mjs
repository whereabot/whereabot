// Wordmark lockup and the 1200x630 Open Graph image. Text is outlined from a bundled font,
// so the output does not depend on fonts installed on the machine that runs the script.
import { readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import opentype from 'opentype.js';
import sharp from 'sharp';
import { COLOURS, MARK, SOCIAL, WORDMARK, at } from './paths.mjs';

const fontFile = createRequire(import.meta.url).resolve('@fontsource/inter/files/inter-latin-700-normal.woff');
const font = opentype.parse(new Uint8Array(readFileSync(fontFile)).buffer);
const body = (file) => readFileSync(file, 'utf8').replace(/^.*?<\/title>/s, '').replace('</svg>', '').trim();
// shortcut: glyph by glyph, no kerning or ligatures (opentype.js cannot shape this font's GSUB); fine for two short strings.
function outline(text, x, y, size, fill) {
  let pen = x;
  const d = [...text].map((ch) => {
    const glyph = font.charToGlyph(ch);
    const data = glyph.getPath(pen, y, size).toPathData(1);
    pen += (glyph.advanceWidth * size) / font.unitsPerEm;
    return data;
  });
  return `<path fill="${fill}" d="${d.join('')}"/>`;
}

// Lockup: the 128-unit mark, then the name.
function wordmarkSvg() {
  const name = outline(SOCIAL.title, 140, 84, 72, COLOURS.text);
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 500 128" width="500" height="128"><title>${SOCIAL.title}</title>${body(MARK)}${name}</svg>\n`;
}

function socialSvg() {
  const lines = SOCIAL.tagline.map((line, i) => outline(line, 120, 450 + i * 60, 48, COLOURS.muted)).join('');
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 630"><rect width="1200" height="630" fill="${COLOURS.background}"/><g transform="translate(120 110) scale(1.9)">${body(WORDMARK)}</g>${lines}</svg>`;
}

export async function renderSocialImage() {
  writeFileSync(WORDMARK, wordmarkSvg());
  await sharp(Buffer.from(socialSvg())).png().toFile(at('site', 'og-image.png'));
}
