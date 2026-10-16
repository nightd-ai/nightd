# Design

## Context

`apps/web` is a minimal Solid + daisyUI + Tailwind + Vite SPA ("Hello World"). The pi monorepo (local clone at `~/code/pi`) provides:

- `@earendil-works/pi-durable` — durable harness with a portable `SqliteStorage` core driven by a small `SqliteDatabase` facade (`exec`/`run`/`get`/`all`/`transaction`/`close`, all async, transactions exclusive). Adapters exist for Node (`node:sqlite`) and Cloudflare Durable Objects; **no browser adapter exists yet**. A browser smoke script proves the runtime-neutral entry points bundle with `platform: "browser"` and catches accidental Node imports.
- `@earendil-works/pi-ai` — `radiusProvider()` targets `https://radius.pi.dev/v1` (`pi-messages` API) with `envApiKeyAuth` falling back to a stored credential from the injected `CredentialStore`; `createModels({ credentials })` accepts an app-owned store. Verified: the gateway sends `access-control-allow-origin: *` and permits `authorization`/`content-type` preflight headers, so browser-to-gateway works directly. (OpenCode Go was probed and rejects browser preflights — see Alternative.)
- The durable harness's tools are opt-in: coding tools only appear if `CodingTools` is explicitly installed. No `ExecutionEnv` is needed when no built-in file/process tools exist.

Constraints that shape the design:

- The SQLite-WASM OPFS VFS needs SharedArrayBuffer, so the page must be cross-origin isolated (`COOP: same-origin`, `COEP: require-corp`).
- Browser dev domain: mkcert is installed; `nightd.localhost` needs a cert and Vite HTTPS wiring.
- Repo conventions: this is the nightd monorepo (pnpm, oxfmt, oxlint type-aware, per-workspace `typecheck`/`test`); commits follow conventional commits.

## Goals / Non-Goals

**Goals:**
- A working end-to-end loop: paste key → prompt → streamed answer → committed session survives reload.
- All harness state durable in OPFS via SQLite-WASM; admissible mid-turn recovery semantics inherited from the harness core.
- Key material that is never exportable and ciphertext-only at rest.

**Non-Goals:**
- WebAuthn PRF-based key derivation (evaluated later; current key is non-extractable via `extractable: false` only).
- Coding tools, execution environments, code mode, MCP, subagents/tasks UI.
- Production deployment concerns (only the Vite dev-serving path is in scope).
- OpenCode Go / any non-Radius provider support in the UI.

## Decisions

1. **Storage adapter as `SqliteDatabase` facade over a Worker** — rather than a hand-written OPFS `Storage` from scratch or wa-sqlite.
   The portable `SqliteStorage` core already implements migrations, cursors, transactions semantics; only the driver is missing. Model: run `@sqlite.org/sqlite-wasm` with the OPFS VFS inside a dedicated Worker (its supported configuration), expose a small MessagePort protocol, and implement the facade with the same `SerialOperationQueue` pattern `node.ts` uses to honor the exclusive-transaction contract. Alternative discarded: wa-sqlite IndexedDB VFS (avoids COOP/COEP but is not the requested OPFS-on-SQLite and is an additional, less-proven dependency).

2. **Credential store = injected `CredentialStore` backed by IndexedDB** — rather than intercepting headers or patching auth.
   Generate the AES-GCM key with `extractable: false`, store the `CryptoKey` object itself in IndexedDB (persistable and non-exportable), encrypt the API key with it, and expose `read`/`list`/`modify`/`delete`. `models.login("radius", "api_key", { prompt })` returns the credential from the UI-stored value. Alternative discarded: WebAuthn PRF (stronger guarantee, per-session user presence — explicitly deferred by the user).

3. **Radius gateway as the only model path** — rather than OpenCode Go or Z.ai direct.
   Empirically verified CORS (`ACAO: *`); OpenCode Go's endpoint fails preflight (404, no CORS headers), which would require a proxying server and violates the no-server constraint. The provider's `pi-messages` API is fetch-based and bundles browser-clean per the existing browser smoke checks.

4. **Weather tool via open-meteo as a custom extension** — rather than a built-in or another API.
   `defineTool` + `defineExtension({ name: "weather", tools: [weather] })`, plain `fetch` to open-meteo (CORS-open, keyless, no execution env needed). `CodingTools` is simply never installed, so no coding tool behavior can surface.

5. **Default agent pinned to `glm-5.3-flash` on the `radius` provider** — created via the harness root options rather than a settings UI. Model selection UI is out of scope.

6. **Vite as the dev delivery** — `server.https` with the mkcert cert/key for `nightd.localhost`, explicit `host`, and `server.headers` for COOP/COEP. No proxy is needed because the gateway is CORS-open from the browser directly.

7. **Self-contained asset strategy under COEP** — all assets local (Tailwind/daisyUI/Solid already local); no third-party scripts/fonts/CDNs so `require-corp` cannot be violated by the page itself. The direct CORS fetch to `radius.pi.dev` is unaffected.

## Risks / Trade-offs

- [OPFS VFS + SharedArrayBuffer browser quirks] → Spike first: verify crossOriginIsolated + OPFS mount in the browser at dev start, before building the full adapter. Fallback if the OPFS VFS misbehaves: SQLite-WASM with a persistence layer over IndexedDB (graceful degradation; OPFS remains the target).
- [Worker-hosted SQLite communication overhead / protocol bugs] → The facade contract is narrow; transaction exclusivity is validated against `storage/sqlite/migrations.ts` and the `SqliteStorage` transaction path with dedicated adapter tests (memory-VFS-backed WebCrypto in Node for logic, browser smoke for the OPFS mount).
- [Non-extractable-in-IDB key still usable by same-origin scripts (XSS can operate the crypto; it cannot extract the key)] → Accepted for this experiment by user decision; WebAuthn PRF listed as future hardening, deliberately out of scope.
- [Radius key tied to a personal/org account in a browser] → Key is revocable; ciphertext-only at rest; no key material in plaintext storage. Accepted experimentally.
- [pi-durable is experimental; API without notice between releases] → Pin exact package versions in `apps/web/package.json`; stay on the published 1.2.0 line.
- [mkcert/loopback resolution depends on the user's resolver setup] → Verify `*.localhost` resolution (systemd-resolved typically handles it) and add a hosts-file fallback only if needed, with user confirmation per system-change policy.

## Migration Plan

No migration: `apps/web` is replaced in place; the previous app was a placeholder. Rollback = git revert. One-time machine setup (mkcert cert issuance) is documented in tasks and is not part of the app.

## Open Questions

None blocking. Deferred deliberately: WebAuthn PRF evaluation (later change), model-selection UI.
