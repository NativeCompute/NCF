# NativeCloud in an app (`ncf::cloud`)

`ncf::cloud` connects a desktop app to NativeCloud, the account and sync service every NativeCompute app shares. It's behind the `cloud` feature and isn't part of the web build.

```toml
ncf = { git = "https://github.com/NativeCompute/ncf", tag = "v0.5.0", features = ["cloud"] }
```

![Settings → Account](shots/cloud-dark.png)

The server, its API and its rules live in [NativeCompute/nativecloud](https://github.com/NativeCompute/nativecloud). This page covers the app side.

## Turning it on
Call `shell.enable_cloud(ctx, CloudConfig::nativecloud())` in your `make` closure.
- If the build has no NativeCloud address, nothing changes and the call returns `false`.
- Once it's on, Settings gains **Account** and **Organizations** tabs, written by NCF:
  - **Sign in with your browser** and **Create an account**. Both open the web panel; the app never shows a password field (see below);
  - while it waits: a countdown, **Open the page again**, **Cancel**, and a box for the 8-character code the web shows when the browser can't reach the app;
  - signed in: your name, email and My cloud meters, **Sync my settings**, this computer, **Manage account on the web** and **Sign out**;
  - invites for you, with **Join** and **Decline**;
  - organizations: your role, plan, spaces and people (read only), a quick invite for owners and admins, and **Manage members on the web**.
- The address comes from `cloud::NATIVECLOUD_URL` and `cloud::NATIVECLOUD_ANON_KEY`. `NCF_CLOUD_URL` and `NCF_CLOUD_ANON_KEY` override them, for staging and tests. The anon key is public by design.
- The web panel is `cloud::NATIVECLOUD_WEB_URL` (`https://cloud.nativecompute.net`). `NCF_CLOUD_WEB_URL` overrides it.

## Signing in through the browser
Sign-in, sign-up, email codes, passwords, profile changes, computers, export and account deletion all happen on the web. The app follows [RFC 8252](https://www.rfc-editor.org/rfc/rfc8252): an authorization code with PKCE over a loopback redirect.
1. `Account::sign_in_with_browser(signup)` makes a `Pkce` (a random verifier, its SHA-256 challenge and a random state) and listens on `127.0.0.1` on a free port for five minutes.
2. It opens `{web}/login?app=…&device=…&challenge=…&state=…&port=…` (with `&mode=signup` for a new account).
3. After signing in, the person confirms "Sign in to Blackline on Ann's Mac?". The web asks the server for a one-time grant code tied to the challenge, then sends the browser to `http://127.0.0.1:{port}/callback?code=…&state=…`.
4. The app checks `state`, answers with a small NCF-styled page, and calls `Client::redeem_code(code, verifier)`. The `app-token` function checks the verifier against the challenge and returns a new session for this computer.
- A wrong `state` is ignored and the app keeps waiting. A code works once, for two minutes, and five wrong verifiers burn it.
- If the browser can't reach the app, the web shows the code instead. `Account::set_code` and `submit_code` redeem it with the same verifier.
- `Client::sign_in` with a password stays for tests and the live CI job only.

**Manage on the web.** `Account::open_on_web(path)` asks the `web-link` function for a one-time link (`{web}/auth/handoff#t=…&next=…`) that signs the browser in as the same person and opens `path`. Signed out, it opens the plain page. `next` is always a path on the panel.

What the app reads back:

| Call | Gives |
|---|---|
| `shell.cloud_client()` | A `Client` for the signed-in account, or `None` when NativeCloud is off. Clones share one session. |
| `shell.cloud.as_mut().and_then(Account::take_changed)` | `Some(true)` after a sign-in or any organization change, and `Some(false)` after sign-out. Reload your cloud lists when it fires. |
| `NcfApp::synced_settings` | The app's own settings keys that **Sync my settings** may carry between computers. Appearance keys are always included. Never list paths or anything that belongs to one computer. |

## The client
`Client` holds the session and makes typed calls that match the server's API:
- `me`, `spaces`, `orgs`, `create_org`, `invite_member`, `projects(app)`, `open_project`, `save_project`, `take_snapshot`, `changes_since`, and the rest.
- Errors are `CloudError`. Every variant has a `title()` and a `message()` written for people, so a notice can show them as they are.
- **Sessions:**
  - the refresh token is kept in the system keychain on macOS and Windows, and in a `0600` file in the app's folder on Linux;
  - an access token refreshes itself a minute before it expires, or after a `401`;
  - a revoked session turns into `CloudError::SignedOut`.
- **The server's clock:** `server_now()` follows the `date` header, so lease expiry is judged by the server's clock rather than the computer's.

## The engine
`Engine<M>` runs calls on a background thread, so the interface never waits on the network:
- `engine.run(|c| M::from(c.me()))` answers later as `Event::Reply(m)` from `engine.poll()`.
- Saves go through an outbox. `engine.save(Pending::new(project, base_version, body))` sends now, or after reconnecting.
- **The outbox:**
  - It persists on disk when it is made with `Outbox::load(path)`.
  - Several saves to one script keep the base version of the first, so only the newest text goes up.
  - Quick saves in a row rebase onto the versions this computer just made, so they never conflict with themselves.
- **When a save can't land**, nothing is lost:
  - **Someone else changed the script meanwhile:** theirs stays, yours is kept as an "offline" snapshot, and `Event::Kept { theirs, snapshot, .. }` tells the app.
  - **Someone else holds the lock:** yours becomes a snapshot, and `Event::Kept { lease, .. }` says who holds it.
- `engine.hold(project)` renews that script's lock every 30 seconds. `Event::LeaseLost { lease }` says who took it over.
- **The change feed** (with `Heartbeat::sync()`):
  - it's polled every 30 seconds, or every 5 seconds while `engine.watch(Some(project))` is set;
  - `Event::Feed(feed)` lists what moved;
  - `Event::Resync` means the app was away too long and should reload everything.
- `Event::State(SyncState)` updates the readout. `view::readout(ui, &state, now)` draws it: *SYNCED · 2S AGO*, *SYNCING · 2*, *OFFLINE · 3 WAITING*, *SIGNED OUT*.

## Testing
With the `testing` feature, `cloud::fake::FakeCloud` is an in-memory NativeCloud that follows the server's rules:
- accounts with codes;
- organizations, spaces, roles, guests and invites;
- browser sign-in grants, `app-token` and `web-link`;
- versions, locks, snapshots, trash and the change feed;
- limits;
- account deletion.

It also has handles a test needs:
- `set_offline`;
- `advance(seconds)`, to move its clock;
- `edit_as` and `lock_as`, to act as another person;
- `code(kind, email)`, to read the emailed code;
- `approve(email, app, device, challenge)`, to play the web's part of a browser sign-in;
- `fail_next`, `requests` and `count`.

`Engine::inline` runs an engine on the test's own thread, and `fake::demo(info)` builds a signed-in account with an organization, two spaces, an admin, a guest and invites both ways. The gallery's **Cloud** page draws from it.
