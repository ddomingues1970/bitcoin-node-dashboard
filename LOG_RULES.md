# Conversation Log Rules — Bitcoin Node Dashboard

Status: Optional protocol; inactive unless the user explicitly issues reglog on.
Timezone: America/Sao_Paulo.

## Activation

- reglog on starts a new conversation log.
- reglog off records the final event and stops logging.
- The command reglog without an argument does not change logging state.
- Reading this document does not activate logging.

## Location and naming

When active, create a new file at:

    logs/bitcoin-node-dashboard_YYYYMMDD_HHMMSS.log

Use America/Sao_Paulo local time. Never overwrite an existing log.

## Entry format

Use concise, one-line entries:

    [YYYY-MM-DD HH:MM:SS -03:00] User: summary
    [YYYY-MM-DD HH:MM:SS -03:00] Assistant: summary

Indent continuation lines by two spaces.

## Privacy

Never log RPC credentials, cookie contents or paths, environment values, wallet names or data, peer IP addresses, or raw node responses. Keep build output, test reports, and screenshots in separate artifacts. Conversation logs are summaries, not operational evidence.

