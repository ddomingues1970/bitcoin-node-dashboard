# Executor Prompt — Add a Safe Dashboard Shutdown Button

You are the Executor making a focused feature change to Bitcoin Node Dashboard.

Work only in:

    .

All code, UI text, documentation, logs, and the completion report must be in US English.

## Read first

Read these project files before editing:

1. AGENTS.md
2. README.md
3. PROJECT_STATUS.md
4. docs/implementation-plan.md
5. docs/architecture.md
6. docs/threat-model.md
7. docs/operator-guide.md
8. LOG_RULES.md for the optional logging protocol only

Do not activate conversation logging unless the Operator explicitly types reglog on.

## Objective

Add a clearly labeled dashboard button that shuts down only the Bitcoin Node Dashboard web server. The button must require a browser confirmation step and the server must stop gracefully, equivalent to pressing Ctrl+C in its terminal.

This feature must never stop, restart, reconfigure, or send commands to Bitcoin Core, Tor, Bisq, or any other system service.

## Security requirements

- Preserve the fixed 127.0.0.1 bind address.
- Do not add CORS or any remote control capability.
- Add a POST-only shutdown route. Do not allow shutdown through GET or browser navigation.
- Protect the action against cross-site request forgery. Use a per-process unpredictable token generated from the operating system's cryptographic randomness, keep it out of the status API, and expose it only to the same-origin dashboard response with no-store caching. Require the token in a custom request header.
- Validate the request Origin against the dashboard's own loopback origin and reject missing or foreign origins. Add any additional same-origin protections that fit the current server design.
- Do not put a reusable shutdown secret in static files, the repository, or logs.
- Make the shutdown signal idempotent and use Axum's graceful-shutdown mechanism. Do not terminate the process abruptly from a request handler.
- The browser must use safe DOM APIs. Show a confirmation dialog before sending the shutdown request.
- After shutdown is accepted, show a clear English message that the dashboard has stopped and the page should be closed.
- Requests with missing/invalid token or wrong Origin must not stop the server.

## Implementation and verification

- Inspect the current routes, application state, listener, signal handling, and UI structure before changing them.
- Add tests for rejecting missing/invalid tokens and foreign/missing Origin headers, and accepting an authorized same-origin request without affecting unrelated services.
- Test the graceful-shutdown signal path without leaving a server process running.
- Update README.md, docs/operator-guide.md, docs/architecture.md, docs/threat-model.md, CHANGELOG.md, and PROJECT_STATUS.md so they describe the new button, confirmation, protection, and stop behavior accurately.
- Run cargo fmt --all -- --check, cargo clippy --all-targets -- -D warnings, cargo build, cargo test, and cargo audit if installed. Report network or tooling blockers clearly.
- Do not commit changes unless the Operator separately asks.

## Completion report

Report the implementation, changed files, validation results, residual risks, and how to use the button. Confirm explicitly that the shutdown route stops only the dashboard and leaves Bitcoin Core, Tor, and Bisq untouched.

