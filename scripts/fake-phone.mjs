// Fake phone for the local e2e. Connects to the relay and answers each locate ping.
// Node's global WebSocket cannot send an Authorization header, so it loads `ws` from WS_PREFIX.
import { createRequire } from 'node:module';

const { WS_URL, PHONE_TOKEN, WS_PREFIX } = process.env;
if (!WS_URL || !PHONE_TOKEN || !WS_PREFIX) {
  console.error('WS_URL, PHONE_TOKEN and WS_PREFIX must be set');
  process.exit(1);
}

const require = createRequire(`${WS_PREFIX}/index.js`);
const WebSocket = require('ws');

const ws = new WebSocket(WS_URL, { headers: { Authorization: `Bearer ${PHONE_TOKEN}` } });

ws.on('open', () => console.log('connected'));
ws.on('message', (data) => {
  const msg = JSON.parse(data.toString());
  if (msg.type !== 'locate') return;
  ws.send(
    JSON.stringify({
      type: 'location',
      id: msg.id,
      latitude: 32.0853,
      longitude: 34.7818,
      accuracy_m: 5,
      captured_at_ms: Date.now(),
    }),
  );
  console.log(`answered ${msg.id}`);
});
ws.on('error', (err) => {
  console.error(`error: ${err.message}`);
  process.exit(1);
});
ws.on('close', () => process.exit(0));
