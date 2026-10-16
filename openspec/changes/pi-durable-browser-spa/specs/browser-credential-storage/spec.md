# Spec Delta

## Purpose

The capability spec delta for keeping the model API key out of reach: a non-extractable encryption key and ciphertext storage for the gateway credential in the browser.

## ADDED Requirements

### Requirement: Non-exportable encryption key
The app SHALL generate an AES-GCM key with `extractable: false` via WebCrypto, and SHALL store that key object in IndexedDB so it persists across sessions while remaining non-exportable.

#### Scenario: Key persists but cannot be exported
- **WHEN** the app reloads after key generation
- **THEN** the stored key is usable for cryptographic operations but attempts to extract its raw material are rejected

#### Scenario: Missing key cannot decrypt
- **WHEN** the stored ciphertext is read but the encryption key is absent (for example, cleared storage)
- **THEN** decryption fails and the app treats the credential as unusable rather than storing plaintext

### Requirement: Encrypted credential at rest
The app SHALL encrypt the API key with the non-exportable key before writing it to IndexedDB, and SHALL never write the plaintext key to IndexedDB, localStorage, or any other persistent storage.

#### Scenario: Encrypted write and read
- **WHEN** the user saves an API key and the app later needs it for model access
- **THEN** the stored value is decrypted in memory only for the request path and removed when no longer needed

#### Scenario: Inspecting storage
- **WHEN** a user (or script) reads the IndexedDB credential record directly
- **THEN** the API key value is obtained only as encrypted bytes

### Requirement: Credential exposed through the model client
The app SHALL make the stored credential available to the pi-ai model layer by implementing the `CredentialStore` interface, so that model requests authenticate with the decrypted key without the key ever being stored in plaintext.

#### Scenario: Model request uses stored credential
- **WHEN** the user submits a prompt after saving an API key
- **THEN** model requests to the gateway carry the decrypted credential taken through the credential store, and no plaintext key is newly persisted

#### Scenario: Credential update and replacement
- **WHEN** the user replaces the stored API key
- **THEN** the previous credential is overwritten with the new encrypted value on the next write
