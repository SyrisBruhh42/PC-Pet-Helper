# LFS Pet — offline Linux family review package

This package is the original local GTK woodland pet and fixed browser-navigation
shell from `SyrisBruhh42/PC-Pet-Helper`. It contains no donor artwork, Rust core,
telemetry, account session, keyring material, checkout metadata, configuration,
backup directories or dependency binaries. The browser owns sign-in and every
Family Hub permission. This package neither reads nor grants those permissions.

## Provenance and dependencies

`manifest.json` records the Git base revision, whether allowlisted source has local
changes, the source-relative path/size/SHA256 of each file and a content-derived
package identifier. Local changes are not represented as committed source.
The whole zip has a separate SHA256 returned by the builder. Obtain that expected
digest from the trusted source handoff, independently of the zip. Digests establish
content identity; this package is not signed. No project release license was
present at the reviewed base. Code/artwork licensing and distribution clearance
remain open; this family review package grants no new licensing rights.

Use Linux, Python 3.9+, GTK 3.24, PyGObject, Pycairo and GI Cairo integration already
provided by the chosen desktop environment. See `docs/LINUX-PET.md` for the native
dependency guide and honest launch diagnostics. Dependencies are not bundled,
downloaded or installed. A virtual environment that hides system GI packages is
not automatically compatible. Choose the existing system Python that exposes GI;
the normal default is `/usr/bin/python3`. Native distro/Wayland qualification is
still open. The installer uses Python's standard library only.

## Verify and install deliberately

Close an existing pet before changing its next-launch version. Keep the verified
archive offline. From the reviewed source checkout, use:

```sh
python3 -B clients/linux/package.py build --output /tmp/lfs-pet-review.zip
python3 -B clients/linux/package.py verify /tmp/lfs-pet-review.zip --expected-sha256 "SHA256_FROM_TRUSTED_HANDOFF"
python3 -B clients/linux/package.py install /tmp/lfs-pet-review.zip --expected-sha256 "SHA256_FROM_TRUSTED_HANDOFF" --python /usr/bin/python3
```

Replace the quoted digest placeholder with the exact returned trusted digest.
For an offline transfer, verify the whole archive digest before extracting its
fixed files into a new owned review directory. The same commands are available
through the reviewed extracted `package.py` for verify/install/activate/remove/
recover; `build` is checkout-only. Never run a fetched URL as shell code. Inspect
source and provenance before executing an unsigned installer.

Installation is per user. It creates only `~/.local/share/lfs-pet/` and the managed
menu entry `~/.local/share/applications/org.lewisfamilysystems.Pet.desktop`, plus
missing parent directories. Existing unfamiliar entries/namespaces, symlinks,
hard-linked files, unsafe ownership or writable ancestors cause a hold. Existing
system Python symlinks are resolved to a trusted executable file; the resulting
absolute interpreter is recorded in the launcher. No root install is supported.
No desktop session, browser, pet, system service or automatic startup is launched
by installation. Find **LFS Pet** in the desktop application menu to launch it
deliberately. The launcher uses direct Python arguments, with bytecode caching
disabled; it never runs a shell. Ordinary title-bar/taskbar controls remain native.

`--home "/absolute/user home"` selects an explicit owned per-user test/installation
home. Spaces, quotes, dollar signs, backticks and backslashes are escaped using the
[Desktop Entry specification](https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html),
not shell quoting. This bounded implementation holds
paths containing controls, non-ASCII characters or `%` rather than claiming
universal path/desktop portability. It uses `.local/share` beneath that home and
does not inspect or rewrite XDG/environment configuration.

## Update, revert, remove and recover

Install a new independently verified zip with the same `install` command. Existing
versions are never overwritten or deleted. A changed retained file causes a hold.
The desktop entry changes future launches only; a running pet is not killed or
implicitly upgraded. Record the returned full `version` identifier for reversal.

```sh
python3 -B clients/linux/package.py activate "PREVIOUS_64_CHARACTER_PACKAGE_ID" --python /usr/bin/python3
python3 -B clients/linux/package.py remove
python3 -B clients/linux/package.py recover
```

`activate` restores a verified retained version's menu entry. `remove` removes only
the exact managed launcher and records an inactive state. Versions, manifests,
history and any interrupted-operation evidence remain. It does not terminate a
running process or remove unrelated core data, settings or backups. Deliberate
purging of retained evidence is outside this bounded command set.

Each activation/removal first durably records the old and intended launcher/state
in `pending.json` and `history/`. If interrupted, new operations hold. `recover`
explicitly restores the before-transaction launcher/state only when observed files
match that transaction; unfamiliar edits/symlinks are preserved and held. It keeps
both package versions and a recovery receipt. Before restoring an active launcher,
it checks all prior package files and the recorded interpreter checksum/custody.
An interpreter update can therefore hold recovery for explicit review. Matching
recovery receipts support retry after interruption; changed receipts hold. Valid
inactive prior states restore without a launcher. Newly created installation
directories sync their pinned containing parent before dependent writes.
An interrupted initial payload copy
can leave an incomplete retained directory; no activation is claimed and that
directory is held for explicit review rather than silently replaced.

## Evidence boundaries

The disposable packaging journey checks the exact allowlist/digests, spaces and
Exec quoting, install/update/retained-version activation, launcher removal,
interrupted activation/recovery-receipt replay, inactive-state recovery, parent
directory syncing and collision/symlink/foreign-owner/tamper rejection. Its native
Gio option launches only a temporary argument-recording fixture, never the pet or
a browser. This is source/function evidence, not owner installation or pet use.

Actual family launch/click/keyboard and accessible desktop-menu behavior remain
open, alongside screen readers, contrast/reduced motion/scaling, GTK/distro/Wayland
matrices, signing/licensing, native credential/state integration, Android devices,
recovery qualification, sustained use and the full adopted pet features. Main,
Aegis, MacroPet, Action Loop and Phoenix retain their independent roles and holds.
