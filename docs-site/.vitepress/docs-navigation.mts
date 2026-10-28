// The landing page is outside the VitePress base. withBase('/../') gives /docs/../, which the browser resolves to /.
export const nav = [
  { text: 'Get started', link: '/getting-started' },
  { text: 'Reference', link: '/reference/mcp-tools' },
  { text: 'Back to Whereabot', link: '/../', target: '_self' },
]

export const sidebar = [
  { text: 'Get started', items: [{ text: 'Connect your agent', link: '/getting-started' }] },
  {
    text: 'Guide',
    items: [
      { text: 'Architecture', link: '/architecture' },
      { text: 'Protocol', link: '/protocol' },
      { text: 'Request flow', link: '/request-flow' },
      { text: 'Operations', link: '/operations' },
      { text: 'Always allow', link: '/always-allow' },
      { text: 'Prior art', link: '/prior-art' },
      { text: 'Roadmap', link: '/roadmap' },
      { text: 'Contributing', link: '/contributing' },
    ],
  },
  {
    text: 'Reference (generated)',
    items: [
      { text: 'MCP tool', link: '/reference/mcp-tools' },
      { text: 'Wire messages', link: '/reference/wire-messages' },
    ],
  },
]
