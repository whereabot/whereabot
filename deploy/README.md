# Deploy to Fly.io

Run these commands from `server/`, where `fly.toml` lives.

1. Install flyctl with `brew install flyctl`, then run `fly auth login`.
2. Run `fly launch --no-deploy --copy-config`. Keep the app name `whereabouts`.
3. Run `fly secrets set WHEREABOUTS_AGENT_TOKEN=$(openssl rand -hex 32) WHEREABOUTS_DEVICE_TOKEN=$(openssl rand -hex 32)`.
4. Write the agent and device tokens down. `fly secrets` cannot show them again.
5. Run `fly deploy`.
6. Test with `curl https://whereabouts.fly.dev/healthz`. It must print `ok`.
