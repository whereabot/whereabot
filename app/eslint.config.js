import recommendedIncremental from 'eslint-config-agent/recommended-incremental'

// Warn-level on-ramp: the strict presets report 690 pre-existing violations in
// this app, so every rule lands as a warning and `eslint .` still exits 0.
// Burn the backlog down, then move to 'eslint-config-agent/recommended'.
export default [
  // Test files sit outside tsconfig (node --test runs them), so the typed parser cannot load them.
  { ignores: ['dist/**', 'dist-web/**', 'src-tauri/**', 'src/**/*.test.ts'] },
  ...recommendedIncremental,
]
