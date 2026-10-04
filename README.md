# PC-Pet-Helper — desktop companion candidate

## Source baseline — 2026-10-04

The canonical source candidate is the current core lineage, based on
`5d6e686339f61688be6fc7c1801302d325a5aac4`. It contains a standalone
state/vitals engine, SQLite persistence, and local state broadcast. It does not
yet display a desktop pet. The aggregate and component branches remain
preserved as unqualified feature material; they have not been merged into this
candidate. See [source decision and acceptance gaps](docs/SOURCE_BASELINE.md).

For the source checks, install the pinned Rust toolchain from
`rust-toolchain.toml`, then run `cargo fmt --all -- --check`,
`cargo build --workspace --locked`, and `cargo test --workspace --locked`.
The same checks are defined in GitHub Actions. Passing them qualifies the core
source only; native desktop behavior and installation remain open.

## Current direction and evidence — 2026-10-03

GitHub metadata observed on 2026-10-03 at 15:43 UTC reports this repository **unarchived**, at `6390a841351dd8c676f2f4e998d2509a0639932c` on `jules-companion-core-impl-11879245625704920277`. The current direction considers a native pet as a candidate primary LFS interface. This repository is retained candidate material; that direction does not establish it as the selected or qualified implementation.

The September 14 no-adoption evaluation below remains historical evidence about that prototype, not the current repository archive flag. Preserve its failures, missing privacy/accessibility/recovery evidence, component references and branches. Reassess claims against current source before reuse; do not infer installation, service startup or credential access from renewed interest.

The remaining product decision is which implementation and supported desktop experience will own the outcome. Runtime, input safety, asset rights, accessibility, packaging and recovery need actual acceptance evidence. Accurate documentation can be updated while those decisions and checks remain open.

## Preserved September 14 evaluation — historical record

# PC-Pet-Helper — archived integration prototype

> **Do not install or treat this repository as a qualified desktop companion.**

Status: archived for reference on 2026-09-14. Pull request 1 and its component branches were preserved, not merged or deleted.

## Why it was archived

The repository contains useful Rust experiments for a companion state machine, telemetry collectors, a CLI/dialogue layer, and a Wayland renderer. It does not yet form an accepted product:

- the default branch contains only the companion-core slice, while the aggregate workspace exists on a separate unqualified PR head;
- there is no product/adoption decision, operator guide, privacy contract, accessibility journey, recovery model, or hosted CI;
- telemetry and window collectors need explicit consent, minimization, retention, redaction, and failure behavior;
- the render path and bundled sprite assets lack native KDE/Wayland acceptance and documented provenance/licensing;
- the systemd/install surface lacks clean install, upgrade, uninstall, rollback, and privilege evidence;
- headless/unit evidence cannot establish real desktop lifecycle, display integration, input safety, or customer experience;
- the design duplicates companion work already integrated into the Action Loop product and proposed for the LFS companion, increasing lifecycle and support variance.

Archiving is a reversible scope decision, not a claim that every component is defective. The retained concepts should be compared against the in-process Action Loop MacroPet and the LFS desktop companion contract before any selective port.

## Ideas intentionally retained

- deterministic pet-state and vitals transitions;
- separation of core, render, telemetry, types, and CLI concerns;
- offline/deterministic dialogue fallback;
- headless renderer checks and explicit desktop-lifecycle tests;
- opt-in resource/status signals that never capture personal content by default.

Any revival needs one chosen product home, a privacy and accessibility contract, asset rights, safe telemetry defaults, native desktop acceptance, packaging/uninstall/rollback proof, and a maintained CI/release pipeline. Avoid creating another autonomous companion runtime unless it provides a clearly distinct, owned user outcome.

## Recovery record

- Default component head before this warning: `c6684d98b4f667ce170e5ecf542b4d9062a0f4d9`
- Aggregate PR head: `b7792449bad45da0f4219b033e47e8ad0a700e80`
- Other component heads: `a46a2422930b5c6b47fce23aa706d69a9503e404`, `ec0e08efd9bdb776e85cb9df3717bbba39d0ef22`, and `b5b6f25d59f13485b49eb325beb796e95d1777fa`
- Private mirror and verified bundle retained through at least `2026-10-14T15:15:37Z`
- GitHub branches and PR discussion remain preserved; no component branch is deleted by this archive action.

<!-- lfs-alignment:begin v1 -->
## LFS alignment

Project role: Native desktop-pet prototype under renewed companion consideration.

This project keeps its own purpose and required features while sharing useful LFS practices: clear ownership, reusable capabilities, scoped access, evidence-based validation and recoverable work. Adopting these practices does not make this repository a deployed LFS service.

Use this README and local project documentation for scope, setup and status. Contributor guidance is in [AGENTS.md](AGENTS.md). A policy update is not proof of tested or delivered functionality.
<!-- lfs-alignment:end -->
