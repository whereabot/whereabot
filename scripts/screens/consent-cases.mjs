import { FAKE } from './playwright-browser.mjs';

export const ORIGIN = 'http://site.local';
export const AUTH_ID = 'fake-authorization-id';
export const AUTH_API = `${FAKE.supabaseUrl}/auth/v1/oauth/authorizations/`;
export const HOLD = 'hold'; // never answered, keeps the page in its pending state

const DETAILS = {
  authorization_id: AUTH_ID,
  redirect_uri: 'https://agent.example.test/callback',
  client: { name: 'Example Agent' },
  user: { email: 'user@example.test' },
};
export const OK = { status: 200, body: DETAILS };
export const APPROVED = { status: 200, body: { redirect_url: 'https://agent.example.test/callback?code=fake-code' } };
const SERVER_ERROR = { status: 500, body: { error: 'server_error' } };
const REJECTED = { status: 400, body: { error: 'invalid_request' } };

/** name -> file suffix. first/then: body state or 'details'/'consent' request to wait for. */
export const CASES = [
  { name: 'loading', query: true, session: true, details: HOLD, first: 'details', desc: 'Checks the request. Status shows Loading with a spinner.' },
  { name: 'signin', query: true, session: false, first: 'signin', desc: 'Signed out. Shows the Apple and Google sign-in buttons.' },
  { name: 'request', query: true, session: true, details: OK, first: 'request', desc: 'Signed in. Shows the app name, the redirect host, Approve and Deny.' },
  { name: 'approving', query: true, session: true, details: OK, consent: HOLD, first: 'request', click: 'approve', then: 'consent', desc: 'Approve pressed. Both buttons are disabled while the answer is sent.' },
  { name: 'error-missing-id', query: false, session: false, first: 'error', desc: 'Opened without authorization_id. Shows the restart message.' },
  { name: 'error-details', query: true, session: true, details: SERVER_ERROR, first: 'error', desc: 'Request details failed to load. Shows the restart message.' },
  { name: 'error-approve', query: true, session: true, details: OK, consent: REJECTED, first: 'request', click: 'approve', then: 'error', desc: 'Approve failed. Shows the restart message and no buttons.' },
  { name: 'error-not-configured', query: true, session: false, configured: false, first: 'signin', click: 'signin-btn', then: 'error', desc: 'Config still has placeholders. Sign-in is blocked with a message.' },
];
