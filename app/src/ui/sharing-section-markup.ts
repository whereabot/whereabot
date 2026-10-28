/** The Sharing screen. Built here, not in index.html, to keep each file small. Wired in sharing-screen.ts. */
export const SHARING_MARKUP = `
        <section id="sharing" class="stack" aria-labelledby="sharing-title">
          <button id="sharing-back" class="btn btn-ghost btn-back" type="button">&larr; Back</button><h2 id="sharing-title" tabindex="-1">Sharing</h2>
          <p>Let someone's AI agent ask where you are. They can ask, and you approve each time unless you choose Always allow. Stop sharing any time.</p>
          <form id="share-form" class="stack" novalidate>
            <div class="field"><label for="share-email">Share my location with&hellip;</label><input id="share-email" type="email" inputmode="email" autocomplete="off" autocapitalize="none" spellcheck="false" placeholder="name@example.com" aria-describedby="share-hint share-error" /></div>
            <p id="share-hint" class="muted">Use the email they sign in to Whereabot with.</p><p id="share-error" class="muted" role="alert"></p>
            <button id="share-submit" class="btn btn-primary" type="submit">Share</button>
          </form>
          <p id="sharing-status" class="muted" role="status" aria-live="polite"></p><button id="sharing-retry" class="btn btn-secondary" type="button" hidden>Try again</button>
          <h3>People I share with</h3><ul id="shared-by-me" class="list"></ul><p id="by-me-empty" class="muted" hidden>You are not sharing with anyone.</p>
          <h3>People sharing with me</h3><ul id="shared-with-me" class="list"></ul><p id="with-me-empty" class="muted" hidden>Nobody shares their location with you yet. Ask them to share it in the Whereabot app.</p>
        </section>
`;

/** Puts the section in front of History, inside the signed-in area. */
export function mountSharingSection() {
  if (document.getElementById("sharing")) return;
  document.getElementById("history")?.insertAdjacentHTML("beforebegin", SHARING_MARKUP);
}
