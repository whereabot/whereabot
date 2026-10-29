// Decides whether a regenerated file matches the committed one.
// PNG resampling (Tauri, sharp) differs by a level or two between CPU architectures, so PNGs compare by
// pixels with a small tolerance; every other file must match byte for byte.
import sharp from 'sharp';

const TOLERANCE = 2; // max difference per channel, out of 255

const pixels = (bytes) => sharp(bytes).raw().toBuffer({ resolveWithObject: true });

export async function sameOutput(file, a, b) {
  if (a === undefined || b === undefined) return a === b;
  if (a.equals(b)) return true;
  if (!file.endsWith('.png')) return false;
  const [x, y] = await Promise.all([pixels(a), pixels(b)]);
  if (x.info.width !== y.info.width || x.info.height !== y.info.height || x.info.channels !== y.info.channels) return false;
  return x.data.every((v, i) => Math.abs(v - y.data[i]) <= TOLERANCE);
}
