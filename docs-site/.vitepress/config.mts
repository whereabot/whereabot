import { defineConfig } from 'vitepress'
import { sidebar, nav } from './docs-navigation.mts'
import { externalizeInlineScripts } from './externalize-inline-scripts.mts'

export default defineConfig({
  title: 'Whereabot Docs',
  description: 'Let your AI agent ask where you are. Architecture, protocol and reference.',
  lang: 'en-US',
  base: '/docs/',
  srcDir: 'generated-source',
  cleanUrls: true,
  head: [['link', { rel: 'icon', href: '/docs/mark.svg', type: 'image/svg+xml' }]],
  themeConfig: {
    logo: '/mark.svg',
    nav,
    sidebar,
    search: { provider: 'local' },
    socialLinks: [{ icon: 'github', link: 'https://github.com/whereabot/whereabot' }],
  },
  buildEnd: externalizeInlineScripts,
})
