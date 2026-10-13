# Headless region probe recipe

Source review only, pinned `bee18c118dbc89f42319436dcdb172d5b9e15e0c`. Paths below are relative to `vendor/OpenNOW/native/opennow-core/src/`. No runtime credential files were accessed.

An independent `--data-dir` avoids the GUI's `core.lock`, which is exclusively locked per directory at `main.rs:78`. It does **not** isolate Linux credentials. `credential_vault.rs:15`, `:132`, and `:791` use the shared OS Secret Service entry `service=app.opennow.auth`, `user=session:<userId>`, irrespective of data directory. `accounts.json` contains identity metadata and active account selection, not session tokens, `credential_vault.rs:20`. Copying it, `settings.json`, and `device-identity.json` can restore through the same unlocked gamer session/keyring without exporting secrets. This only works if credentials are in the OS store. File fallback under `fallback-sessions/` is directory-specific and deliberately excluded.

Preserve `console-profiles-qt.json` too if present. `console_profiles.rs:70` owns PIN state; omitting it loses PIN gating. `gfn.rs:952` suppresses automatic restoration for accounts with a PIN. Avoid copying session/cleanup files, locks, legacy auth-state, or fallback-session secrets. `device_identity.rs:23` validates the copied durable identity.

Use newline-delimited JSON requests, correlated by `id`:

```json
{"type":"request","id":"1","method":"core.hello","params":{"protocolVersion":5,"shell":"qt","shellVersion":"0.5.4"}}
{"type":"request","id":"2","method":"auth.accounts.list","params":{}}
{"type":"request","id":"3","method":"auth.session.get","params":{}}
{"type":"request","id":"4","method":"network.regions.list","params":{}}
{"type":"request","id":"5","method":"network.regions.ping","params":{"regions":[{"name":"<returned name>","url":"<returned url>"}]}}
```

Handlers are at `main.rs:392`, `:546`, `:562`, `:692`. Account listing returns `{accounts,activeUserId,generation}`, `gfn.rs:1282`; activeUserId can be null before restoration. `auth.session.get` lazily restores active credentials and returns a token-free `{session,generation,persistence,refresh,warnings,deviceIdentity}` envelope, `gfn.rs:919`. For another saved account use `auth.accounts.switch` with `{userId}` and optional `{pin}`, `gfn.rs:1303`. Region listing returns `{regions:[{name,url}],vpcId,providerIdpId}`, `gfn.rs:1559`, `:3329`.

`network.rs:137` accepts at most 32 region URL objects and returns `{results:[{url,pingMs,error?}]}` in input order. It does one TCP warmup, then three connects 100 ms apart and reports their rounded mean, `network.rs:173`. Port follows the URL, normally HTTPS 443. This is TCP connection latency, not UDP stream latency, loss, jitter, decode or presentation performance. Repeat externally and aggregate medians.

Concurrency caveat: both session restore and authenticated region discovery may refresh tokens automatically within the 10-minute access or 5-minute client-token windows, `gfn.rs:67`, `:989`. Refresh writes the **shared keyring**, `gfn.rs:2824`; its mutex is process-local. Two cores can race token rotation and leave the GUI holding an old token. Separate profile directories do not solve this. A sidecar is therefore not provably read-only even with only these methods. The safest active-stream probe is an unauthenticated sidecar calling only `core.hello` and `network.regions.ping` with URLs already obtained through the existing authenticated core. For independent authenticated discovery, wait for a session boundary and serialize credential use; do not call logout, account removal or session mutations.
