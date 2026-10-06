#!/usr/bin/env python3
"""One disposable-user packaging/update/reversal/recovery journey; no pet/browser."""

import argparse
from contextlib import redirect_stderr, redirect_stdout
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import zipfile

import package


def held(callback):
    try:
        callback()
    except (package.PackageError, OSError, ValueError):
        return
    raise AssertionError("Expected an explicit hold, never silent replacement")


def command(arguments):
    output, errors = io.StringIO(), io.StringIO()
    with redirect_stdout(output), redirect_stderr(errors):
        status = package.main(arguments)
    assert status == 0, errors.getvalue()
    return json.loads(output.getvalue())


def journey(native=False):
    source = Path(__file__).absolute().parents[2]
    revision = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    root = Path(tempfile.mkdtemp(prefix="lfs-pet-package-check-"))
    fixture = root / "source fixture"
    for relative in package.FILES.values():
        package.write_new(fixture / relative, package.read_source(source / relative))
    # Deliberately fake excluded material, not real accounts, configuration or credentials.
    package.write_new(fixture / ".git/never-package", b"synthetic checkout metadata")
    package.write_new(fixture / "backups/provider.secret", b"SYNTHETIC-NOT-A-SECRET")
    package.write_new(fixture / "donor-assets/not-used.svg", b"synthetic omitted asset")
    # Instrument the single journey: every newly created directory must fsync
    # its pinned parent before another filesystem operation can depend on it.
    original_mkdir, original_fsync = package.os.mkdir, package.os.fsync
    awaiting_parent = []
    def checked_mkdir(path, mode=0o777, *, dir_fd=None):
        assert not awaiting_parent, "Previous mkdir parent was not synced"
        value = original_mkdir(path, mode, dir_fd=dir_fd)
        assert dir_fd is not None, "Installer mkdir must use a pinned parent"
        awaiting_parent.append(dir_fd)
        return value
    def checked_fsync(fd):
        if awaiting_parent:
            assert awaiting_parent.pop() == fd, "Wrong mkdir containing parent synced"
        return original_fsync(fd)
    package.os.mkdir, package.os.fsync = checked_mkdir, checked_fsync
    first = package.build_package(fixture, root / "first.zip", revision, True)
    same = package.build_package(fixture, root / "same.zip", revision, True)
    assert same["sha256"] == first["sha256"]
    manifest, payload = package.verify_package(first["artifact"], first["sha256"])
    assert manifest["source"] == {"repository": package.REPOSITORY, "revision": revision, "dirty": True}
    assert set(payload) == set(package.FILES)
    with zipfile.ZipFile(first["artifact"]) as archive:
        assert set(archive.namelist()) == set(package.FILES) | {"manifest.json"}
    for name, relative in package.FILES.items():
        assert payload[name] == package.read_file(fixture / relative)
        assert manifest["files"][name]["sha256"] == package.digest(payload[name])
    held(lambda: package.verify_package(first["artifact"], "0" * 64))
    held(lambda: package.build_package(fixture, root / "first.zip", revision, True))
    assert package.digest(package.read_file(root / "first.zip")) == first["sha256"]
    # Independently checks that archive traversal/extra files fail before extraction.
    unsafe = root / "unsafe.zip"
    with zipfile.ZipFile(unsafe, "w") as archive:
        archive.writestr("../../not-an-installed-file", "SYNTHETIC")
    held(lambda: package.verify_package(unsafe, package.digest(package.read_file(unsafe))))

    home = root / 'Family home $cash `tick` "quote" \\slash'
    original_mkdir(home, 0o700)
    recorder = root / "fake python"
    arguments = root / "native-arguments.json"
    recorder.write_text("#!/usr/bin/python3\nimport json, sys\n"
                        f"with open({str(arguments)!r}, 'w', encoding='utf-8') as stream:\n"
                        "    json.dump(sys.argv[1:], stream)\n", encoding="utf-8")
    recorder.chmod(0o700)
    installation = package.Installation(home)
    foreign = b"[Desktop Entry]\nType=Application\nName=Unrelated fixture\n"
    package.write_new(installation.desktop, foreign)
    held(lambda: installation.install(manifest, payload, recorder))
    assert package.read_file(installation.desktop) == foreign
    installation.desktop.unlink()  # Explicit fixture cleanup, never installer takeover.
    installed = command(["install", first["artifact"], "--expected-sha256", first["sha256"], "--home", str(home), "--python", str(recorder)])
    assert installed["version"] == first["package_id"]
    version1 = installation.version(first["package_id"])
    desktop1 = package.read_file(installation.desktop)
    assert b"Type=Application\n" in desktop1 and b"Terminal=false\n" in desktop1
    assert b"Autostart" not in desktop1 and b"sh -c" not in desktop1 and b"%" not in desktop1
    assert b" -B " in desktop1 and desktop1.decode().count("Exec=") == 1
    expected_arguments = ["-B", str(version1 / "lfs_pet.py")]
    if native:
        from gi.repository import Gio
        app = Gio.DesktopAppInfo.new_from_filename(str(installation.desktop))
        assert app is not None
        assert app.launch([], None) is True  # Only the owned argument recorder, no GTK pet/browser.
        until = time.monotonic() + 3
        while not arguments.exists() and time.monotonic() < until:
            time.sleep(0.02)
        assert json.loads(arguments.read_text(encoding="utf-8")) == expected_arguments
        native_result = "PASS_DISPOSABLE_GIO_ARGUMENT_RECORDER"
    else:
        native_result = "NOT_RUN_GIO_NATIVE_OPTION"
    # Fixed independent specification vector: literal backslash needs four,
    # quoted special characters two backslashes in the Desktop Entry string.
    vector = desktop1.decode().split("Exec=", 1)[1].splitlines()[0]
    assert 'Family home \\\\$cash \\\\`tick\\\\` \\\\"quote\\\\" \\\\\\\\slash' in vector

    guide = fixture / "clients/linux/PACKAGE.md"
    guide.write_bytes(guide.read_bytes() + b"\nSynthetic package update fixture.\n")
    second = package.build_package(fixture, root / "second.zip", revision, True)
    assert second["package_id"] != first["package_id"]
    updated = command(["install", second["artifact"], "--expected-sha256", second["sha256"], "--home", str(home), "--python", str(recorder)])
    assert updated["version"] == second["package_id"]
    installation.version(first["package_id"])
    installation.version(second["package_id"])
    desktop2 = package.read_file(installation.desktop)
    assert desktop2 != desktop1
    assert command(["activate", first["package_id"], "--home", str(home), "--python", str(recorder)])["version"] == first["package_id"]
    assert package.read_file(installation.desktop) == desktop1
    command(["activate", second["package_id"], "--home", str(home), "--python", str(recorder)])

    # One injected activation interruption after desktop replacement, before state commit.
    original_replace = package.replace_file
    def interrupted(path, data, expected):
        if Path(path) == installation.root / "active.json":
            raise OSError("SYNTHETIC_INTERRUPTED_ACTIVATION")
        return original_replace(path, data, expected)
    package.replace_file = interrupted
    try:
        held(lambda: installation.activate(first["package_id"], recorder))
    finally:
        package.replace_file = original_replace
    assert package.read_file(installation.root / "pending.json")
    held(lambda: installation.remove())
    # Corrupted before-version and changed interpreter both hold without restore.
    prior_payload = installation.root / "versions" / second["package_id"] / "navigation.py"
    saved_payload = prior_payload.read_bytes()
    prior_payload.write_bytes(saved_payload + b"\n# synthetic prior corruption\n")
    held(lambda: installation.recover())
    assert package.read_file(installation.desktop) == desktop1
    prior_payload.write_bytes(saved_payload)
    saved_interpreter = recorder.read_bytes()
    recorder.write_bytes(saved_interpreter + b"\n# synthetic changed interpreter\n")
    held(lambda: installation.recover())
    assert package.read_file(installation.desktop) == desktop1
    recorder.write_bytes(saved_interpreter)
    # Crash after recovered receipt creation, before exact pending removal.
    def interrupted_recovery(path, data, expected):
        if Path(path) == installation.root / "pending.json" and data is None:
            raise OSError("SYNTHETIC_INTERRUPTED_RECOVERY_RECEIPT")
        return original_replace(path, data, expected)
    package.replace_file = interrupted_recovery
    try:
        held(lambda: installation.recover())
    finally:
        package.replace_file = original_replace
    journal = json.loads(package.read_file(installation.root / "pending.json"))
    recovery_file = installation.root / "history" / journal["receipt"] / "recovered.json"
    saved_recovery = recovery_file.read_bytes()
    recovery_file.write_bytes(saved_recovery + b" ")
    held(lambda: installation.recover())
    assert package.read_file(installation.root / "pending.json")
    recovery_file.write_bytes(saved_recovery)
    assert command(["recover", "--home", str(home)])["recovered"] is True
    assert package.read_file(installation.desktop) == desktop2
    assert installation.state()[0]["version"] == second["package_id"]
    assert list((installation.root / "history").glob("*/recovered.json"))
    assert command(["remove", "--home", str(home)])["version"] is None
    assert not installation.desktop.exists()
    installation.version(first["package_id"])
    installation.version(second["package_id"])
    # A valid inactive prior state is also recoverable without creating a launcher.
    package.replace_file = interrupted
    try:
        held(lambda: installation.activate(second["package_id"], recorder))
    finally:
        package.replace_file = original_replace
    assert command(["recover", "--home", str(home)])["recovered"] is True
    assert not installation.desktop.exists() and installation.state()[0]["version"] is None
    victim = root / "unrelated victim.txt"
    package.write_new(victim, b"PRESERVE_UNRELATED_FIXTURE")
    installation.desktop.symlink_to(victim)
    held(lambda: installation.activate(first["package_id"], recorder))
    assert installation.desktop.is_symlink() and package.read_file(victim) == b"PRESERVE_UNRELATED_FIXTURE"
    installation.desktop.unlink()  # Only explicit disposal of this synthetic attack link.
    command(["activate", second["package_id"], "--home", str(home), "--python", str(recorder)])
    retained = version1 / "navigation.py"
    retained.write_bytes(retained.read_bytes() + b"\n# synthetic tamper, held and preserved\n")
    held(lambda: installation.activate(first["package_id"], recorder))
    assert package.read_file(installation.desktop) == desktop2
    assert b"synthetic tamper" in package.read_file(retained)
    unsafe_home = root / "unsafe writable home"
    original_mkdir(unsafe_home, 0o700)
    unsafe_home.chmod(0o777)
    held(lambda: package.Installation(unsafe_home))
    unsafe_home.chmod(0o700)
    unsupported = root / "percent%home"
    held(lambda: package.Installation(unsupported))
    unmarked_home = root / "unmarked home"
    original_mkdir(unmarked_home, 0o700)
    unmarked = unmarked_home / ".local/share/lfs-pet"
    package.write_new(unmarked / "preserve.txt", b"SYNTHETIC_UNMANAGED_NAMESPACE")
    held(lambda: package.Installation(unmarked_home).install(manifest, payload, recorder))
    assert package.read_file(unmarked / "preserve.txt") == b"SYNTHETIC_UNMANAGED_NAMESPACE"
    command(["remove", "--home", str(home)])
    # Foreign-owned safe-mode ancestor simulated by metadata injection only;
    # never chown the host or require privileges.
    foreign_dir = root / "foreign ancestor"
    with package.directory(foreign_dir, create=True, user_leaf=True):
        pass
    child = foreign_dir / "child"
    with package.directory(child, create=True, user_leaf=True):
        pass
    assert package.Installation(child).home == child  # Safe owned control must exist and succeed.
    original_fstat = package.os.fstat
    foreign_inode = foreign_dir.stat().st_ino
    def foreign_fstat(fd):
        info = original_fstat(fd)
        if info.st_ino == foreign_inode and info.st_dev == foreign_dir.stat().st_dev:
            fields = list(info)
            fields[4] = 424242
            return package.os.stat_result(fields)
        return info
    package.os.fstat = foreign_fstat
    try:
        try:
            package.Installation(child)
        except package.PackageError as error:
            assert "unsafe directory owner" in str(error)
        else:
            raise AssertionError("Foreign-owned ancestor was admitted")
        with package.directory(foreign_dir, source_read=True):
            pass
    finally:
        package.os.fstat = original_fstat
    assert not awaiting_parent
    package.os.mkdir, package.os.fsync = original_mkdir, original_fsync
    result = {"status": "PASS_DISPOSABLE_PACKAGE_JOURNEY", "journey_root": str(root), "native_desktop": native_result,
              "versions_retained": 2, "recovery_receipt": True, "pet_browser_core_started": False,
              "holds": ["wrong-digest", "archive-overwrite", "archive-traversal", "foreign-desktop", "interrupted-activation", "symlink-desktop", "tampered-retained-version", "writable-home", "unsupported-path", "unmarked-namespace", "foreign-owned-ancestor", "prior-payload-recovery", "prior-interpreter-recovery", "changed-recovery-receipt"],
              "foreign_ancestor_safe_control": "PASS", "foreign_ancestor_package_error": "PASS", "mkdir_parent_sync": "PASS", "idempotent_recovery": "PASS", "inactive_prior_recovery": "PASS",
              "scope": "SOURCE_AND_DISPOSABLE_FUNCTION_ONLY"}
    package.write_new(root / "journey-result.json", package.canonical(result))
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-desktop-check", action="store_true", help="Use installed Gio to launch only a disposable argument recorder")
    options = parser.parse_args()
    print(json.dumps(journey(options.native_desktop_check), sort_keys=True))
