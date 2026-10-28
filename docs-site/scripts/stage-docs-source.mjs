// Builds generated-source/ (gitignored) from the repo markdown. Nothing here is committed.
import { copyFileSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { rewriteRepoLinks } from './rewrite-repo-links.mjs';

const SITE = join(dirname(fileURLToPath(import.meta.url)), '..');
const ROOT = join(SITE, '..');
const OUT = join(SITE, 'generated-source');

rmSync(OUT, { recursive: true, force: true });
mkdirSync(join(OUT, 'public'), { recursive: true });

const docs = readdirSync(join(ROOT, 'docs')).filter((name) => name.endsWith('.md'));
for (const name of docs) {
  const text = readFileSync(join(ROOT, 'docs', name), 'utf8');
  writeFileSync(join(OUT, name), rewriteRepoLinks(text, 'docs', docs));
}

// The user-facing setup part of the README becomes the "Get started" page.
const readme = readFileSync(join(ROOT, 'README.md'), 'utf8');
const section = readme.match(/^## Connect your agent\n([\s\S]*?)(?=^## )/m);
if (!section) throw new Error('README.md has no "## Connect your agent" section');
writeFileSync(join(OUT, 'getting-started.md'), rewriteRepoLinks(`# Get started\n${section[1]}`, '', docs));

copyFileSync(join(SITE, 'pages', 'index.md'), join(OUT, 'index.md'));
copyFileSync(join(ROOT, 'site', 'mark.svg'), join(OUT, 'public', 'mark.svg'));
console.log(`Staged ${docs.length} docs plus home and get-started into ${OUT}`);
