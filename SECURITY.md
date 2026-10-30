# Security policy

Whereabot handles live location. Report a vulnerability privately. Never open a public issue for it.

## Report a vulnerability

1. Go to the [Security tab](https://github.com/whereabot/whereabot/security) and click **Report a vulnerability** (GitHub private vulnerability reporting).
2. Describe the issue, the affected component (relay, app, database, site) and the steps to reproduce.
3. Do not include real location data or other people's accounts in the report.

If the Security tab has no report button, open an issue that asks for a private contact and contains no details.

## What happens next

- A maintainer replies within 7 days.
- We fix the issue, then publish an advisory with credit, unless you ask us not to name you.
- Please give us 90 days, or until a fix ships, before you disclose.

## Scope

- In scope: this repository and the hosted service at whereabot.io and relay.whereabot.io.
- Out of scope: denial of service by load, social engineering, and issues in your own self-hosted configuration.
- Test only with accounts and phones that you own.

## Supported versions

Only the latest `main` and the latest app release get fixes.

Security design: [architecture, security](docs/architecture.md#security). Data handling: [privacy](docs/privacy.md).
