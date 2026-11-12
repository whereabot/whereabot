import recommendedIncremental from 'eslint-config-agent/recommended-incremental'

// Warn-level on-ramp, same as app/ and scripts/: every rule lands as a warning
// so `npm run lint` exits 0 while the backlog stays visible. Burn it down, then
// move to 'eslint-config-agent/recommended'.
export default [
  { ignores: ['.vitepress/dist/**', '.vitepress/cache/**', 'generated-source/**'] },
  ...recommendedIncremental,
]
