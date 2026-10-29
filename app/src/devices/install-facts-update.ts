/** No imports, so `node --test` can load it as plain TS. */
type Update = (fields: Record<string, string>) => PromiseLike<unknown>;

/**
 * One best-effort update per column: each column has its own migration, and a client can deploy
 * before one runs. A combined update would fail whole on 42703 and drop the other column too.
 */
export async function sendInstallFacts(update: Update, facts: { app_kind: string; screen_hint: string }) {
  await Promise.allSettled(Object.entries(facts).map(([k, v]) => update({ [k]: v })));
}
