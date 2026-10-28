import { defineConfig } from 'vitepress'
import { sidebar, nav } from './docs-navigation.mts'
import { externalizeInlineScripts } from './externalize-inline-scripts.mts'

export default defineConfig({
  // Docs use <Name> placeholders in prose; render raw HTML as text so Vue never parses it.
  markdown: { html: false },
  title: 'Whereabot Docs',
  description: 'Let your AI agent ask where you are. Architecture, protocol and reference.',
  lang: 'en-US',
  base: '/docs/',
  srcDir: 'generated-source',
  cleanUrls: true,
  head: [
    ['link', { rel: 'icon', href: '/docs/favicon.ico', sizes: '48x48' }],
    ['link', { rel: 'icon', href: '/docs/mark.svg', type: 'image/svg+xml' }],
    ['link', { rel: 'apple-touch-icon', href: '/docs/apple-touch-icon.png' }],
  ],
  themeConfig: {
    logo: '/mark.svg',
    nav,
    sidebar,
    search: { provider: 'local' },
    socialLinks: [{ icon: 'github', link: 'https://github.com/whereabot/whereabot' }],
  },
  buildEnd: externalizeInlineScripts,
})
