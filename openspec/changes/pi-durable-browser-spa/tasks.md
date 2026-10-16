# Tasks

## 1. Dev environment and app shell

- [ ] 1.1 Issue a mkcert TLS cert for `nightd.localhost`, verify `nightd.localhost` resolves to loopback on this machine (systemd-resolved or hosts entry — ask before any system change), and confirm `https://nightd.localhost` serves the current app; verify with `curl -I https://nightd.localhost` showing a valid chain
- [ ] 1.2 Replace the Hello World app: strip App.tsx to a layout shell, add COOP/COEP headers and HTTPS cert wiring in `apps/web/vite.config.ts`, and verify in the browser that `crossOriginIsolated === true` on `https://nightd.localhost` (spike from design Risks — gate for the storage work)

## 2. Browser storage adapter

- [ ] 2.1 Add dependencies (`@earendil-works/pi-durable`, `@earendil-works/pi-ai`, `@earendil-works/chord`, `@sqlite.org/sqlite-wasm`, pinned versions) and verify `pnpm install` and `pnpm --filter web typecheck` pass
- [ ] 2.2 Implement the Worker hosting SQLite-WASM with the OPFS VFS (open/mount, exec/run/get/all/message protocol, close) and verify a smoke message round-trip (open a throwaway DB, run `SELECT 1`, dispose) in the browser
- [ ] 2.3 Implement the `SqliteDatabase` facade in the page (call queue, exclusive transactions with commit/rollback, prepared-call batching) and verify transaction semantics: concurrent op is queued, rejected callback rolls back
- [ ] 2.4 Wire `SqliteStorage` over the facade and verify the migrations apply and a harness can write/read a conversation; identity check: reopen after reload and observe the same session data (see browser storage spec scenarios)

## 3. Encrypted credential store

- [ ] 3.1 Implement non-extractable key generation + IndexedDB persistence (`extractable: false`, unit test in Node/WebCrypto or component test asserting persists-without-export semantics in the browser)
- [ ] 3.2 Implement the encrypted credential record (AES-GCM encrypt/decrypt of the API key, keyed-IDB read/write) with unit tests: write, read, decrypt; ciphertext-only at rest; missing-key read fails cleanly
- [ ] 3.3 Wrap it as a pi-ai `CredentialStore` (read/list/modify/delete) and verify `models.login("radius", "api_key", …)` stores through it and `getAuth` resolves an auth result without plaintext persistence

## 4. Harness wiring

- [ ] 4.1 Wire the harness module: `createModels({ credentials })` + `radiusProvider()`, registry with only the weather extension, `Harness.open` on the OPFS-backed storage, root conversation pinned to `radius`/`glm-5.3-flash`; verify the agent object resolves with no coding tools listed
- [ ] 4.2 Implement the weather tool (`defineTool` + `defineExtension`, plain `fetch` to open-meteo geocoding + forecast) and verify a scripted trigger ("what's the weather in Graz") calls the tool and renders structured weather output
- [ ] 4.3 Verify an end-to-end browser round trip: submit a prompt → assistant answer commits → reload → transcript reappears from OPFS storage (browser storage + UI spec scenarios)

## 5. UI

- [ ] 5.1 Build the key-setup screen (paste + save path feeding the credential store; gates prompting until a usable key exists; re-entry on auth failure) and verify the browser-agent-ui key scenarios manually in the browser
- [ ] 5.2 Build the prompt + transcript view wired to `root.watch()`: entries, streamed `pi.live` partials, tool-call progress/results, usage; verify empty-prompt guard, ordering, streaming display, and tool-call rendering against the browser-agent-ui scenarios
- [ ] 5.3 Run the repo gates: `pnpm run fmt`, `pnpm -r typecheck`, `pnpm run lint`, `pnpm -r test` — all clean

## Workflow follow-up

- Archive the change after the project's review requirements are satisfied.
- Verify the archived result.
- Evaluate WebAuthn PRF key hardening as a follow-up change (out of scope here, recorded in design).
