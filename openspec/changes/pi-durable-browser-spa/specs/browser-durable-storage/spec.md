# Spec Delta

## Purpose

The capability spec delta for durable in-browser storage: SQLite compiled to WASM with its OPFS backing through the pi-durable storage contract.

## ADDED Requirements

### Requirement: Durable session persistence
The app SHALL persist the entire conversation state (transcript entries, submissions, tasks, documents) to a SQLite database stored in the browser's OPFS, such that a page reload does not lose committed state.

#### Scenario: Reload after committing
- **WHEN** the user submits a prompt, the answer has committed, and the page is reloaded
- **THEN** the same session, including the prompt and answer, is loaded from storage and shown

#### Scenario: Mid-keyframe recovery
- **WHEN** a generation task dies mid-turn (browser closed, crash) before an answer commits
- **THEN** reopening the app picks the work up from the durable state without data loss of prior turns

### Requirement: Storage backend implements the durable contract
The browser storage SHALL implement the full `Storage` behavior the pi-durable harness relies on, by running the portable SQLite storage core on an SQLite-WASM database that honors its facade contract: single-statement SQL execution, serialized exclusive transactions, and correct commit/rollback.

#### Scenario: Concurrent operations serialize
- **WHEN** a transaction is running and another storage operation arrives
- **THEN** the foreign operation is executed only after the transaction has committed or rolled back

#### Scenario: Rollback on error
- **WHEN** a transaction callback rejects
- **THEN** the transaction is rolled back and the storage remains in its pre-transaction state

### Requirement: Origin-private storage location
The stored database SHALL live in the browser's OPFS and SHALL be reachable only from the same origin.

#### Scenario: Storage survives across browser sessions
- **WHEN** the user closes and reopens the browser and returns to the app
- **THEN** the previously stored session database is still present for that origin

### Requirement: Serving conditions for storage
The app SHALL be served in development over HTTPS with mkcert-issued TLS on `nightd.localhost`, with COOP/COEP response headers that leave the page cross-origin isolated, because the OPFS SQLite VFS requires SharedArrayBuffer.

#### Scenario: Cross-origin isolated app loads
- **WHEN** the app is opened at `https://nightd.localhost` in development
- **THEN** the page runs with COOP/COEP headers applied and cross-origin isolation can be verified before the storage opens

#### Scenario: Storage opens under isolation
- **WHEN** the SQLite-WASM OPFS backing initializes on the isolated page
- **THEN** it opens without reducing the availability of other features and without third-party scripts that break cross-origin isolation
