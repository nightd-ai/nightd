# Proposal

## Why

The pi Durable harness currently runs only in Node/server contexts. Proving that a coding-agent harness can run entirely client-side — persisted storage, model access, and tool execution in the browser — is the first step toward browser-first agent experiences and validates the portable core of `@earendil-works/pi-durable` outside its native environments.

## What Changes

- Replace the `apps/web` "Hello World" Solid SPA with a single-page app that runs the `@earendil-works/pi-durable` harness entirely in the browser (no server-side code beyond static serving).
- Add a browser SQLite storage adapter: SQLite compiled to WASM with its OPFS VFS in a dedicated Worker, implementing the `SqliteDatabase` facade consumed by `SqliteStorage`.
- Add an encrypted credential store: the Radius API key is stored encrypted in IndexedDB using a non-extractable AES-GCM `CryptoKey` generated in-browser and persisted in IndexedDB itself; the store implements the pi-ai `CredentialStore` interface.
- Access the Radius model gateway (`radius.pi.dev/v1`, CORS-enabled) via the pi-ai `radiusProvider` and target the `glm-5.3-flash` virtual model.
- Install a single weather tool (open-meteo, CORS-enabled, keyless) as a custom extension; coding tools are explicitly not installed.
- Build a UI with a prompt input and a session view that renders transcript entries, streamed response partials, tool-call progress/results, and usage.
- Serve the app in development on `https://nightd.localhost` with mkcert TLS and COOP/COEP headers so the OPFS VFS can use SharedArrayBuffer.

## Capabilities

### New Capabilities

- `browser-agent-ui`: The SPA interface — entering a prompt, viewing the session transcript with live streaming and tool-call progress, and setting up the model API key.
- `browser-durable-storage`: Durable harness storage in the browser — the SQLite-WASM-on-OPFS adapter, its transaction/serialization guarantees, and the serving conditions (TLS dev domain, cross-origin isolation) required for it to work.
- `browser-credential-storage`: Secure storage of the model API key — non-extractable key material, encrypted at rest in IndexedDB, and exposure through the pi-ai credential flow.

### Modified Capabilities

(none — no existing specs)

## Impact

- `apps/web`: full rewrite of `src/App.tsx`, new source modules, updated `vite.config.ts` (HTTPS, headers, worker build).
- New npm dependencies in `apps/web`: `@earendil-works/pi-durable`, `@earendil-works/pi-ai`, `@earendil-works/chord`, `@sqlite.org/sqlite-wasm`.
- Development machine (one-time): mkcert certificate for `nightd.localhost`; `*.localhost` loopback resolution verified.
- pi monorepo: only consumed as published packages (`@earendil-works/*`); no edits required there.
