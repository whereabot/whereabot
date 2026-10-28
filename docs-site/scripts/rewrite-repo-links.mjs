import { posix } from 'node:path';

const GITHUB = 'https://github.com/whereabot/whereabot';
const LINK = /\]\(([^)\s#]+)(#[^)\s]*)?\)/g;

/**
 * Rewrites relative markdown links of a repo file. Links to another staged doc become
 * `./name.md`. Links to anything else in the repo become GitHub links.
 * `fromDir` is the repo folder of the file ("docs" or "" for the README).
 */
export function rewriteRepoLinks(text, fromDir, stagedDocs) {
  return text.replace(LINK, (whole, target, anchor = '') => {
    if (/^([a-z][a-z0-9+.-]*:|\/\/|\/)/i.test(target)) return whole;
    const repoPath = posix.normalize(posix.join(fromDir, target)).replace(/\/$/, '');
    const staged = repoPath.match(/^docs\/([^/]+\.md)$/);
    if (staged && stagedDocs.includes(staged[1])) return `](./${staged[1]}${anchor})`;
    const kind = posix.extname(repoPath) ? 'blob' : 'tree';
    return `](${GITHUB}/${kind}/main/${repoPath}${anchor})`;
  });
}
