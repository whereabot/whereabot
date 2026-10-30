// Viewports for the no-scroll check, and the in-page measure of page scroll and sideways overflow.
const PHONES = [[320, 568], [375, 667], [390, 844], [430, 932]];

export const FIT_VIEWPORTS = [
  ...PHONES.map(([width, height]) => ({ name: `${width}x${height}`, width, height })),
  ...PHONES.map(([height, width]) => ({ name: `${width}x${height}`, width, height })), // landscape
  { name: '768x1024', width: 768, height: 1024 },
  { name: '800x600', width: 800, height: 600 }, // desktop app window default (tauri.conf.json)
  { name: '1280x800', width: 1280, height: 800 },
  { name: '1920x1080', width: 1920, height: 1080 },
];

/** Runs in the page. Returns problems: the page scrolls, or a visible element pokes out sideways. */
export function measureFit() {
  const problems = [];
  const root = document.scrollingElement;
  if (root.scrollHeight > innerHeight + 1) problems.push(`page scrolls: ${root.scrollHeight} > ${innerHeight}`);
  if (root.scrollWidth > innerWidth + 1) problems.push(`page scrolls sideways: ${root.scrollWidth} > ${innerWidth}`);
  for (const node of document.body.querySelectorAll('*')) {
    if (!node.checkVisibility() || node.closest('.sr-only')) continue;
    const box = node.getBoundingClientRect();
    if (box.width && (box.right > innerWidth + 1 || box.left < -1)) {
      const name = `${node.tagName.toLowerCase()}#${node.id}.${node.getAttribute('class')}`;
      problems.push(`wider than the screen: ${name} ${Math.round(box.left)}..${Math.round(box.right)}`);
    }
  }
  return problems;
}
