# Location request flow

Every state a request passes through, every notification the user can see, and every result the agent gets. Wire formats: [protocol](protocol.md). Components: [architecture](architecture.md).

Legend: rounded boxes are what the user sees; hexagons are relay decisions; `[A]` = answer the agent gets.

```mermaid
flowchart TD
    A([Agent calls get_phone_location]) --> R{{Relay: any device<br/>of this user connected?}}

    R -->|"yes, app in front"| P{{Phone: agent always allowed?}}
    R -->|"no / app in background"| W{{Relay: agent always allowed?}}

    %% --- App open in front ---
    P -->|no| PROMPT(["In-app prompt:<br/>'Your agent X wants to know your location'<br/>✕ Reject · ✓ Allow · ★ Always allow"])
    P -->|yes| AUTO_FG["Answer at once, no prompt"]
    AUTO_FG --> BANNER(["In-app banner + history:<br/>'X checked your location'"])
    BANNER --> FIX

    %% --- App in background or closed ---
    W -->|no| ALERT(["Push notification:<br/>'Your agent X wants to know your location'<br/>Reject · Allow · Always allow"])
    W -->|"yes"| SILENT["Silent push wakes the app<br/>(no banner)"]
    SILENT -->|app woke| AUTO_BG["Native code reads one fix<br/>and posts it"]
    AUTO_BG --> CHECKED(["Notification:<br/>'X checked your location'<br/>tap opens History"])
    CHECKED --> FIX
    SILENT -->|"no answer in ~8 s<br/>(force-quit, Low Power)"| CHECKING(["Notification:<br/>'X is checking your location'<br/>Share"])
    CHECKING -->|Share| GETFIX

    ALERT -->|Allow / Always allow| GETFIX
    ALERT -->|Reject| DENIED
    ALERT -->|"tap body: app opens"| PROMPT
    ALERT -->|ignored| TIMEOUT

    PROMPT -->|"✓ Allow"| GETFIX
    PROMPT -->|"★ Always allow<br/>(remembered)"| GETFIX
    PROMPT -->|"✕ Reject"| DENIED
    PROMPT -->|"no tap before expiry"| NOANSWER

    %% --- Reading the location ---
    GETFIX{{Phone: location permission?}} -->|"off / denied"| PERM
    GETFIX -->|granted| READ["Watch up to 10 s,<br/>keep best fix, stop at ≤30 m"]
    READ -->|got a fix| FIX
    READ -->|nothing| NOFIX

    %% --- Results for the agent ---
    FIX[/"[A] latitude, longitude, accuracy_m"/]
    DENIED[/"[A] 'The user declined to share their location.' (403)"/]
    NOANSWER[/"[A] 'The user did not respond on their phone.' (408)"/]
    PERM[/"[A] 'Location access is off for Whereabot…' (424)"/]
    NOFIX[/"[A] 'The phone could not get a location fix.' (424)"/]
    TIMEOUT[/"[A] 'The phone did not answer in time.' (504)"/]
    R -->|"no device within 60 s<br/>and no push answer"| OFFLINE[/"[A] 'The user's phone is not connected…' (503)"/]

    %% --- Other phones of the same user ---
    FIX & DENIED --> CANCEL(["Other phones: prompt closes,<br/>history 'Answered on another device'"])
```

## Notifications at a glance

| When | Title / body | Buttons | Status |
|---|---|---|---|
| Agent not always allowed, app not in front | "Your agent X wants to know your location" | Reject · Allow · Always allow | Live |
| Always-allowed agent, answered in the background | "X checked your location" — "Shared just now. Manage in Whereabot." | none (tap opens History) | Live |
| Always-allowed agent, iOS did not wake the app | "X is checking your location" | Share | Live |
| App open in front | No system notification; in-app prompt or banner | Reject · Allow · Always allow | Live |
| Shared request, not always allowed | "R's agent X wants to know your location" | Reject · Allow · Always allow | New |
| Shared request, always allowed, iOS did not wake the app | "R's agent X is checking your location" | Share | New |

Every outcome is also written to the in-app **Request history**.

## Shared requests

The agent passes `person` to ask for someone who shares their location with its user. The relay first checks the share; it then runs the same flow above on the **owner's** phones. "Your agent X" reads "R's agent X", where R is the requester. "Always allowed" means the owner always allows agent X **for R**. Extra agent results:

- No active share matches `person` (unknown or revoked): "Nobody named P shares their location with you…" (404).
- More than one owner matches: the text lists each "Name (email)" (409).
- The owner rejects: "NAME declined to share their location." (403).

Details: [location sharing](location-sharing.md).
