# PC-Pet source baseline decision — 2026-10-04

This record describes a source candidate, not an installed or accepted desktop
companion. The repository is `SyrisBruhh42/PC-Pet-Helper`. The observed GitHub
default on 2026-10-04 was
`jules-companion-core-impl-11879245625704920277` at
`5d6e686339f61688be6fc7c1801302d325a5aac4`. The repository was
unarchived; no open pull request, workflow definition, or check run was returned
for that head. The existing local checkout outside this candidate was not
changed.

## Canonical source and retained work

Keep the current core lineage as the canonical *source foundation*. It has a
small coherent Cargo workspace: shared pet types, state transitions, vitals,
SQLite persistence, and a Unix socket state stream. It ticks and persists
without LFS, a model provider, window telemetry, or a desktop renderer. A
visible native pet is still a product requirement, not a feature this source
candidate has achieved.

| Retained ref (observed head) | Meaningful content | Disposition and reason |
| --- | --- | --- |
| `jules-companion-core-impl-11879245625704920277` (`5d6e686`) | Core engine, types, focused tests, and dated status guidance | Canonical source foundation. Preserve its history and current default until a separate reviewed default-branch decision. |
| `jules-3830590763270704827-8d9acabd` (`b779244`) | Combined workspace with core, renderer, collectors, CLI, dialogue experiments, service files, and assets | Preserve as an unqualified feature reservoir. Historical PR #1 was closed unmerged. The renderer draws into an in-memory pixmap but does not submit a visible Wayland surface; input-region application is a placeholder. The workspace also starts privacy-sensitive collectors by default. Do not merge this tree wholesale. |
| `jules-9364428966253130273-5034a3b5` (`b5b6f25`) | Sprite physics/animation, images, and headless renderer tests | Preserve. The headless tests do not show display, input, accessibility, or asset-rights acceptance. |
| `jules-10028004132781683069-5f0067fa` (`a46a242`) | CPU, battery, idle, window, and Git activity collectors plus socket server | Preserve. Explicit opt-in, data minimization, retention, authorization, and failure behavior are not established. |
| `jules-11733534774000400264-cb8bf374` (`ec0e08e`) | CLI/orchestrator, deterministic dialogue fallback, optional Ollama call, and systemd installer | Preserve. The dialogue module is not wired into the CLI's active path; the installer enables a service without verified packaging, rollback, or desktop behavior. |

The three individual renderer, telemetry, and CLI histories have no common
ancestor with the current default. The aggregate is one commit based on the
older core commit and copies their components into its tree. Tree content is
the useful comparison here; unrelated-history merging would not qualify the
features and would obscure the acceptance boundary. All remote branches and
their historical tags remain intact.

## Source checks and limits

The source gate is `cargo fmt --all -- --check`, `cargo build --workspace
--locked`, and `cargo test --workspace --locked` with the pinned toolchain.
The new workflow runs these checks on pushes and pull requests. A local build
and 13 existing core tests passed at the observed default code before this
candidate's formatting and workflow changes. Rerun the gate at the exact
candidate commit; a local result does not substitute for GitHub CI.

The current core writes to `~/.local/share/desktop-pet/pet_state.db`. If the
default database cannot open, it logs a warning and continues in memory, so
the user can lose persistence. Its state socket uses a fixed `/tmp` path and
is not a qualified multi-user or privileged IPC boundary. It has no installer,
update, export, uninstall, rollback, or supported-platform matrix. These are
open behavior and lifecycle requirements, not implied by a passing build.

## Standalone behavior and optional LFS connection

The pet must remain useful when LFS is offline, unavailable, denied, or never
configured. Local state transitions, vitals, basic interaction, and an
accessible status/control path must not wait for a remote service. The current
core covers local state/vitals and persistence only; interaction and a visible
or accessible user journey still need implementation and native acceptance.

Before a future integration, the owning LFS provider and pet consumer should
agree on a versioned typed operation contract. A minimal request needs a
provider/consumer identity, operation and resource, user-granted scope,
private-data audience, request ID for deduplication, deadline/cancellation,
and an explicit offline/denied result. For example, a pet status request may
read a scoped family status summary, while a denied request yields a local
explanation and the next safe action; it must not silently fall back to wider
data access. The provider must enforce permissions itself. Network presence
or a prompt is not authorization. This is a contract requirement, not a
shipping API, endpoint, or credential grant. Reuse the existing LFS typed
operation/identity/status contracts when their actual owner and version are
confirmed; do not add another broker for this prototype.

## Acceptance still required

- On a supported KDE/Wayland desktop, show a visible pet and prove its click
  region does not intercept unrelated desktop input; test startup, exit, and
  recovery on the actual session.
- Provide a keyboard/screen-reader accessible status and interaction path,
  plus a usable app/web fallback when native rendering is unavailable.
- Settle telemetry consent and minimization before enabling window titles,
  repository activity, or idle collection. Record retention and redaction.
- Establish provenance and permission for the bundled image assets before
  distribution. Keep uncertainty visible; no license clearance is claimed.
- Qualify install, update, local-data export, uninstall, rollback, and supported
  platform behavior with real receipts. Windows and Apple remain roadmap
  targets rather than tested platforms.
- Verify the exact candidate commit in CI, then review any default-branch
  change separately. Source, CI, native desktop, and release acceptance are
  distinct states.
