import recommendedIncremental from 'eslint-config-agent/recommended-incremental'

// Warn-level on-ramp, same as app/eslint.config.js: the strict preset reports
// 313 pre-existing violations across these build/screenshot scripts, so every
// rule lands as a warning and `npm run lint` still exits 0. Burn the backlog
// down, then move to 'eslint-config-agent/recommended'.
export default [{ ignores: ['brand-assets/out/**'] }, ...recommendedIncremental]
