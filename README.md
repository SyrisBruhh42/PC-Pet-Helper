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
