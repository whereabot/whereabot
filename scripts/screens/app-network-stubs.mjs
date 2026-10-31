import { FAKE } from './playwright-browser.mjs';

// Supabase calls return an empty JSON body. No network leaves the machine.
export const EMPTY_JSON = (route) => route.fulfill({ status: 200, contentType: 'application/json', body: '{}' });

// The relay acks every answer (and its CORS preflight), as the real POST /v1/device/answer does.
const ANSWER_ACK = (route) => route.fulfill({
  status: 204,
  headers: {
    'access-control-allow-origin': '*',
    'access-control-allow-methods': 'POST',
    'access-control-allow-headers': 'authorization,content-type',
  },
});

/** Every outside request the app makes in the screen harness; the rest is aborted. */
export const APP_STUBS = [
  [FAKE.supabaseUrl, EMPTY_JSON],
  ['https://relay.example.test/v1/device/answer', ANSWER_ACK],
];
