import { createClient } from './supabase.esm.mjs';
import { SUPABASE_URL, SUPABASE_ANON_KEY } from './supabase-config.js';

const $ = (id) => document.getElementById(id);
const authorizationId = new URLSearchParams(location.search).get('authorization_id');
// Unfilled config (local preview) still renders the signed-out view. Sign-in is blocked below.
const configured = !SUPABASE_URL.startsWith('__');
const supabase = createClient(configured ? SUPABASE_URL : 'https://unconfigured.invalid', SUPABASE_ANON_KEY);

const SECTIONS = ['signin', 'request'];
function show(id) {
  for (const name of SECTIONS) $(name).hidden = name !== id;
  document.body.dataset.state = id;
}

function fail(message) {
  $('status').textContent = message;
  for (const name of SECTIONS) $(name).hidden = true;
  document.body.dataset.state = 'error';
  $('restart').hidden = false;
}

function busy(on) {
  $('approve').disabled = on;
  $('deny').disabled = on;
}

// Consent stays a user action. Only a click on Approve sends approval.
async function decide(kind) {
  busy(true);
  const call = kind === 'approve' ? supabase.auth.oauth.approveAuthorization : supabase.auth.oauth.denyAuthorization;
  const { data, error } = await call(authorizationId, { skipBrowserRedirect: true });
  if (error) {
    busy(false);
    return fail('This request could not be completed. Start the sign-in again from your app.');
  }
  location.replace(data.redirect_url);
}

async function start() {
  if (!authorizationId) {
    return fail('Missing authorization_id. Start the sign-in again from your app.');
  }
  const { data: session } = await supabase.auth.getSession();
  if (!session.session) {
    $('status').textContent = 'Sign in to see which app asks for your location.';
    return show('signin');
  }
  const { data, error } = await supabase.auth.oauth.getAuthorizationDetails(authorizationId);
  if (error) {
    return fail('This request could not be loaded. Start the sign-in again from your app.');
  }
  // Supabase returns a redirect, not details, when this client already has consent.
  if (!('authorization_id' in data)) return location.replace(data.redirect_url);

  $('client-name').textContent = data.client.name;
  $('redirect-host').textContent = new URL(data.redirect_uri).host;
  $('user-email').textContent = data.user.email;
  $('status').textContent = 'Check the app and the host below, then choose.';
  show('request');
}

// Same account as the phone: Sign in with Apple or Google (data-provider on each button).
document.querySelectorAll('[data-provider]').forEach((btn) => btn.addEventListener('click', async () => {
  if (!configured) return fail('This page is not configured. Deploy fills supabase-config.js.');
  const { error } = await supabase.auth.signInWithOAuth({
    provider: btn.dataset.provider,
    options: { redirectTo: location.href },
  });
  if (error) $('status').textContent = 'Sign-in could not start. Try again.';
}));
$('signout').addEventListener('click', async () => {
  await supabase.auth.signOut();
  location.reload();
});
$('approve').addEventListener('click', () => decide('approve'));
$('deny').addEventListener('click', () => decide('deny'));

start().catch(() => fail('Something went wrong. Reload this page or start again from your app.'));
