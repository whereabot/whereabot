import { createHash } from 'node:crypto'
import { readdirSync, readFileSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import type { SiteConfig } from 'vitepress'

// Inline <script> tags that carry code. Tags with src, or data blocks (type="application/json"), stay.
const INLINE_SCRIPT = /<script(?![^>]*\ssrc=)(?![^>]*type="application\/json")([^>]*)>([\s\S]*?)<\/script>/g

function htmlFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory() ? htmlFiles(join(dir, entry.name)) : entry.name.endsWith('.html') ? [join(dir, entry.name)] : []
  )
}

/**
 * Moves every inline script into /docs/assets/inline.<hash>.js. The CSP can then say
 * script-src 'self' with no hashes and no 'unsafe-inline', and nothing breaks when page content changes.
 */
export function externalizeInlineScripts(config: SiteConfig) {
  const base = config.site.base
  for (const file of htmlFiles(config.outDir)) {
    const html = readFileSync(file, 'utf8')
    const next = html.replace(INLINE_SCRIPT, (_tag, attrs: string, code: string) => {
      const name = `inline.${createHash('sha256').update(code).digest('hex').slice(0, 10)}.js`
      writeFileSync(join(config.outDir, 'assets', name), code)
      return `<script${attrs} src="${base}assets/${name}"></script>`
    })
    if (next !== html) writeFileSync(file, next)
  }
}
