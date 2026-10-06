#!/usr/bin/env python3
"""Offline, allowlisted Linux shell packages and reversible per-user launchers.

No dependency installation, shell commands, browser/API calls or automatic startup.
"""

import argparse
from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import uuid
import zipfile

FORMAT = "lfs-linux-pet-package-v1"
OWNER_FORMAT = "lfs-linux-pet-install-v1"
REPOSITORY = "SyrisBruhh42/PC-Pet-Helper"
FILES = {
    "lfs_pet.py": "clients/linux/lfs_pet.py",
    "navigation.py": "clients/linux/navigation.py",
    "package.py": "clients/linux/package.py",
    "README.md": "clients/linux/PACKAGE.md",
    "docs/LINUX-PET.md": "docs/LINUX-PET.md",
}
MAX_BUNDLE = 2 * 1024 * 1024
MAX_FILE = 1024 * 1024
DESKTOP = "org.lewisfamilysystems.Pet.desktop"


class PackageError(Exception):
    pass


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("utf-8")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def identifier(value):
    if not isinstance(value, str) or not re.fullmatch(r"[a-f0-9]{64}", value):
        raise PackageError("Choose an exact 64-character package/checksum identifier.")
    return value


def absolute(path):
    result = Path(os.path.abspath(os.path.expanduser(str(path))))
    if any(ord(char) < 32 or ord(char) > 126 for char in str(result)) or "%" in str(result):
        raise PackageError("This bounded launcher requires ASCII paths without controls or %. Spaces are supported.")
    return result


@contextmanager
def directory(path, create=False, user_leaf=False, source_read=False):
    """Traverse pinned directory FDs, refusing symlinks and unsafe writable ancestors."""
    path = absolute(path)
    descriptor = os.open("/", os.O_RDONLY | os.O_DIRECTORY)
    try:
        for depth, part in enumerate(path.parts[1:]):
            if create:
                try:
                    os.mkdir(part, 0o700, dir_fd=descriptor)
                    os.fsync(descriptor)
                except FileExistsError:
                    pass
            following = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=descriptor)
            info = os.fstat(following)
            tmp_exception = depth == 0 and part == "tmp" and bool(info.st_mode & stat.S_ISVTX)
            if not source_read and ((info.st_uid not in (0, os.getuid()) and not tmp_exception) or (info.st_mode & 0o022 and not tmp_exception)):
                os.close(following)
                raise PackageError("Refusing an unsafe directory owner or writable ancestor.")
            os.close(descriptor)
            descriptor = following
        if user_leaf and os.fstat(descriptor).st_uid != os.getuid():
            raise PackageError("The installation directory must belong to this user.")
        yield descriptor
    finally:
        os.close(descriptor)


def read_file(path, missing=False, trusted_root=False):
    path = absolute(path)
    try:
        with directory(path.parent) as parent:
            descriptor = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent)
            try:
                info = os.fstat(descriptor)
                owners = (0, os.getuid()) if trusted_root else (os.getuid(),)
                if not stat.S_ISREG(info.st_mode) or info.st_uid not in owners or info.st_nlink != 1 or info.st_mode & 0o022 or info.st_size > MAX_BUNDLE:
                    raise PackageError("Refusing an unsafe file, link, ownership, permissions or size.")
                with os.fdopen(descriptor, "rb", closefd=False) as stream:
                    data = stream.read(MAX_BUNDLE + 1)
                if len(data) > MAX_BUNDLE:
                    raise PackageError("File exceeds the bounded size.")
                return data
            finally:
                os.close(descriptor)
    except FileNotFoundError:
        if missing:
            return None
        raise


def read_source(path):
    """Read one fixed checkout input without treating a shared checkout as an install root."""
    path = absolute(path)
    with directory(path.parent, source_read=True) as parent:
        descriptor = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent)
        with os.fdopen(descriptor, "rb") as stream:
            info = os.fstat(stream.fileno())
            if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_size > MAX_FILE:
                raise PackageError("Source input must be a bounded regular file, not a link.")
            data = stream.read(MAX_FILE + 1)
            if len(data) > MAX_FILE:
                raise PackageError("Source input exceeds the per-file limit.")
            return data


def write_new(path, data):
    path = absolute(path)
    with directory(path.parent, create=True, user_leaf=True) as parent:
        descriptor = os.open(path.name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=parent)
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.fsync(parent)


def replace_file(path, data, expected):
    """Only replace/remove the exact previously observed managed content."""
    path = absolute(path)
    with directory(path.parent, create=True, user_leaf=True) as parent:
        current = read_file(path, missing=True)
        if current != expected:
            raise PackageError("Managed file changed. Preserve it and review recovery; no overwrite was made.")
        if data is None:
            if current is not None:
                os.unlink(path.name, dir_fd=parent)
        else:
            temporary = ".lfs-write-" + uuid.uuid4().hex
            descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=parent)
            with os.fdopen(descriptor, "wb") as stream:
                stream.write(data)
                stream.flush()
                os.fsync(stream.fileno())
            # Protected user-owned parents and the namespace lock exclude other-user writers.
            if read_file(path, missing=True) != expected:
                os.unlink(temporary, dir_fd=parent)
                raise PackageError("Managed file changed during preparation. No replacement was made.")
            os.replace(temporary, path.name, src_dir_fd=parent, dst_dir_fd=parent)
        os.fsync(parent)


def build_package(source, output, revision, dirty):
    source, output = absolute(source), absolute(output)
    if not re.fullmatch(r"[a-f0-9]{40}", revision):
        raise PackageError("A verified Git source revision is required.")
    payload = {name: read_source(source / relative) for name, relative in FILES.items()}
    if any(len(data) > MAX_FILE for data in payload.values()):
        raise PackageError("Allowlisted source exceeds the per-file limit.")
    metadata = {"format": FORMAT, "source": {"repository": REPOSITORY, "revision": revision, "dirty": bool(dirty)},
                "files": {name: {"source_path": FILES[name], "sha256": digest(data), "size": len(data)} for name, data in payload.items()}}
    manifest = {**metadata, "package_id": digest(canonical(metadata))}
    import io
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in sorted({**payload, "manifest.json": canonical(manifest)}.items()):
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | 0o600) << 16
            archive.writestr(info, data)
    bundle = stream.getvalue()
    if len(bundle) > MAX_BUNDLE:
        raise PackageError("Package exceeds the offline bundle limit.")
    write_new(output, bundle)
    return {"artifact": str(output), "sha256": digest(bundle), "package_id": manifest["package_id"], "source": manifest["source"], "files": len(payload)}


def verify_package(path, expected_sha256):
    import io
    bundle = read_file(path)
    if digest(bundle) != identifier(expected_sha256):
        raise PackageError("Package checksum does not match the independently obtained expected digest.")
    with zipfile.ZipFile(io.BytesIO(bundle)) as archive:
        entries = archive.infolist()
        expected = set(FILES) | {"manifest.json"}
        if len(entries) != len(expected) or {item.filename for item in entries} != expected:
            raise PackageError("Package has unexpected, duplicate or missing entries; nothing was extracted.")
        for item in entries:
            if not stat.S_ISREG(item.external_attr >> 16) or item.file_size > MAX_FILE or item.flag_bits & 1:
                raise PackageError("Package links, encrypted entries or oversized files are unsupported.")
        if sum(item.file_size for item in entries) > MAX_BUNDLE:
            raise PackageError("Uncompressed package exceeds the bounded size.")
        payload = {item.filename: archive.read(item) for item in entries}
    manifest = json.loads(payload.pop("manifest.json"))
    if not isinstance(manifest, dict) or set(manifest) != {"format", "source", "files", "package_id"} or manifest["format"] != FORMAT or not isinstance(manifest["source"], dict):
        raise PackageError("Unsupported package manifest.")
    source = manifest["source"]
    if set(source) != {"repository", "revision", "dirty"} or source["repository"] != REPOSITORY or not isinstance(source["dirty"], bool) or not re.fullmatch(r"[a-f0-9]{40}", str(source["revision"])):
        raise PackageError("Unreadable source provenance.")
    if not isinstance(manifest["files"], dict) or set(manifest["files"]) != set(FILES):
        raise PackageError("Unreadable allowlisted file manifest.")
    for name, data in payload.items():
        if manifest["files"][name] != {"source_path": FILES[name], "sha256": digest(data), "size": len(data)}:
            raise PackageError("A file differs from its declared source digest/size.")
    metadata = {key: value for key, value in manifest.items() if key != "package_id"}
    if manifest["package_id"] != digest(canonical(metadata)):
        raise PackageError("Package identity does not match its manifest.")
    return manifest, payload


def desktop_argument(value):
    """Desktop Entry string escaping then Exec argument quoting; never shell quoting."""
    value = str(absolute(value))
    quoted = value.replace("\\", "\\\\").replace('"', '\\"').replace("`", "\\`").replace("$", "\\$")
    return '"' + quoted.replace("\\", "\\\\") + '"'


def interpreter(path):
    path = absolute(Path(path).resolve(strict=True))
    if "=" in str(path):
        raise PackageError("The desktop executable path cannot contain =.")
    with directory(path.parent) as parent:
        descriptor = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent)
        try:
            info = os.fstat(descriptor)
            if not stat.S_ISREG(info.st_mode) or info.st_uid not in (0, os.getuid()) or info.st_mode & 0o022:
                raise PackageError("Choose a trusted, non-writable interpreter file.")
        finally:
            os.close(descriptor)
    if not os.access(path, os.X_OK):
        raise PackageError("Choose an existing executable Python interpreter; no dependency is installed.")
    return str(path)


def interpreter_digest(path):
    path = Path(interpreter(path))
    with directory(path.parent) as parent:
        descriptor = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW, dir_fd=parent)
        with os.fdopen(descriptor, "rb") as stream:
            info = os.fstat(stream.fileno())
            if info.st_uid not in (0, os.getuid()) or not stat.S_ISREG(info.st_mode) or info.st_mode & 0o022 or info.st_size > 32 * 1024 * 1024:
                raise PackageError("Recorded interpreter custody is unavailable or unsafe.")
            checksum = hashlib.sha256()
            for block in iter(lambda: stream.read(65536), b""):
                checksum.update(block)
            return checksum.hexdigest()


def desktop_entry(version, python):
    return ("[Desktop Entry]\nType=Application\nName=LFS Pet\n"
            "Comment=Local family companion; sign in in your browser\n"
            f"Exec={desktop_argument(python)} -B {desktop_argument(version / 'lfs_pet.py')}\n"
            "Terminal=false\nStartupNotify=false\nCategories=Utility;\n").encode("utf-8")


class Installation:
    def __init__(self, home):
        if os.geteuid() == 0:
            raise PackageError("Do not install this per-user package as root.")
        self.home = absolute(home)
        with directory(self.home, user_leaf=True):
            pass
        self.root = self.home / ".local/share/lfs-pet"
        self.desktop = self.home / ".local/share/applications" / DESKTOP
        self.marker = canonical({"format": OWNER_FORMAT, "uid": os.getuid(), "home": str(self.home)})

    @contextmanager
    def locked(self, create=False):
        existing = read_file(self.root / "owner.json", missing=True)
        if existing is None:
            if not create:
                raise PackageError("No managed installation exists at this home.")
            try:
                with directory(self.root):
                    raise PackageError("Existing unmarked package directory is held; no takeover is allowed.")
            except FileNotFoundError:
                pass
            with directory(self.root.parent, create=True, user_leaf=True) as parent:
                os.mkdir(self.root.name, 0o700, dir_fd=parent)
                os.fsync(parent)
            write_new(self.root / "owner.json", self.marker)
        elif existing != self.marker:
            raise PackageError("Installation marker does not match this user/home. Preserve it.")
        with directory(self.root, user_leaf=True) as parent:
            descriptor = os.open(".lock", os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600, dir_fd=parent)
            try:
                info = os.fstat(descriptor)
                if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_nlink != 1 or info.st_mode & 0o022:
                    raise PackageError("Unsafe installation lock.")
                fcntl.flock(descriptor, fcntl.LOCK_EX)
                yield
            finally:
                os.close(descriptor)

    def version(self, package_id):
        root = self.root / "versions" / identifier(package_id)
        manifest = json.loads(read_file(root / "manifest.json"))
        metadata = {key: value for key, value in manifest.items() if key != "package_id"}
        if manifest.get("format") != FORMAT or manifest.get("package_id") != package_id or digest(canonical(metadata)) != package_id or set(manifest.get("files", {})) != set(FILES):
            raise PackageError("Retained version manifest changed. No activation was made.")
        for name in FILES:
            data = read_file(root / name)
            if manifest["files"][name] != {"source_path": FILES[name], "sha256": digest(data), "size": len(data)}:
                raise PackageError("Retained version changed. Preserve it; no activation was made.")
        return root

    def state(self):
        data = read_file(self.root / "active.json", missing=True)
        return (json.loads(data) if data is not None else None), data

    def transition(self, action, package_id=None, python=None):
        if read_file(self.root / "pending.json", missing=True) is not None:
            raise PackageError("An interrupted transaction is retained. Use recover before another operation.")
        before, before_bytes = self.state()
        old_desktop = read_file(self.desktop, missing=True)
        if old_desktop is not None and (before is None or digest(old_desktop) != before.get("desktop_sha256")):
            raise PackageError("Desktop entry is unrecognized or changed; no overwrite/removal was made.")
        if old_desktop is None and before is not None and before.get("desktop_sha256") is not None:
            raise PackageError("Managed desktop entry is missing; review before a new activation.")
        version = self.version(package_id) if package_id is not None else None
        python = interpreter(python) if version is not None else None
        new_desktop = desktop_entry(version, python) if version is not None else None
        after = {"format": OWNER_FORMAT, "version": package_id, "python": python, "python_sha256": interpreter_digest(python) if python else None, "desktop_sha256": digest(new_desktop) if new_desktop else None}
        after_bytes = canonical(after)
        receipt_id = uuid.uuid4().hex
        journal = {"format": OWNER_FORMAT, "action": action, "receipt": receipt_id,
                   "before_state": before_bytes.decode() if before_bytes else None, "after_state": after_bytes.decode(),
                   "before_desktop": old_desktop.decode() if old_desktop else None, "after_desktop": new_desktop.decode() if new_desktop else None}
        # Persist recovery information before either activation file changes.
        write_new(self.root / "pending.json", canonical(journal))
        write_new(self.root / "history" / receipt_id / "prepared.json", canonical(journal))
        replace_file(self.desktop, new_desktop, old_desktop)
        replace_file(self.root / "active.json", after_bytes, before_bytes)
        write_new(self.root / "history" / receipt_id / "completed.json", canonical({"action": action, "state": after}))
        replace_file(self.root / "pending.json", None, canonical(journal))
        return after

    def install(self, manifest, payload, python):
        with self.locked(create=True):
            # Check launcher collision and pending recovery before retaining the new package.
            old, _ = self.state()
            current = read_file(self.desktop, missing=True)
            if read_file(self.root / "pending.json", missing=True) is not None or (current is not None and (old is None or digest(current) != old.get("desktop_sha256"))):
                raise PackageError("Existing launcher or interrupted operation needs review; no activation was made.")
            package_id = manifest["package_id"]
            target = self.root / "versions" / package_id
            if read_file(target / "manifest.json", missing=True) is None:
                with directory(target.parent, create=True, user_leaf=True) as parent:
                    os.mkdir(package_id, 0o700, dir_fd=parent)
                    os.fsync(parent)
                # A partial version is retained on interruption, never silently overwritten.
                for name, data in payload.items():
                    write_new(target / name, data)
                write_new(target / "manifest.json", canonical(manifest))
            self.version(package_id)
            return self.transition("install", package_id, python)

    def activate(self, package_id, python):
        with self.locked():
            return self.transition("activate-retained", identifier(package_id), python)

    def remove(self):
        with self.locked():
            return self.transition("remove-launcher")

    def recover(self):
        with self.locked():
            raw = read_file(self.root / "pending.json")
            journal = json.loads(raw)
            if journal.get("format") != OWNER_FORMAT or not re.fullmatch(r"[a-f0-9]{32}", str(journal.get("receipt"))):
                raise PackageError("Unreadable recovery evidence; preserve it for review.")
            old_desktop = journal["before_desktop"].encode() if journal["before_desktop"] is not None else None
            new_desktop = journal["after_desktop"].encode() if journal["after_desktop"] is not None else None
            old_state = journal["before_state"].encode() if journal["before_state"] is not None else None
            new_state = journal["after_state"].encode()
            # Validate the complete prior launch target before restoring either file.
            if old_state is None:
                if old_desktop is not None:
                    raise PackageError("Missing prior state for recorded launcher.")
            else:
                previous = json.loads(old_state)
                if set(previous) != {"format", "version", "python", "python_sha256", "desktop_sha256"} or previous["format"] != OWNER_FORMAT:
                    raise PackageError("Unreadable prior launch state.")
                if previous["version"] is None:
                    if old_desktop is not None or any(previous[key] is not None for key in ("python", "python_sha256", "desktop_sha256")):
                        raise PackageError("Inactive prior state is inconsistent.")
                else:
                    retained = self.version(previous["version"])
                    if interpreter(previous["python"]) != previous["python"] or interpreter_digest(previous["python"]) != previous["python_sha256"]:
                        raise PackageError("Prior interpreter changed; recovery held.")
                    if desktop_entry(retained, previous["python"]) != old_desktop or digest(old_desktop) != previous["desktop_sha256"]:
                        raise PackageError("Prior launcher does not match retained version/interpreter.")
            recovery = canonical({"action": "restore-before-transaction", "versions": "RETAINED", "journal_sha256": digest(raw)})
            recovery_path = self.root / "history" / journal["receipt"] / "recovered.json"
            existing_recovery = read_file(recovery_path, missing=True)
            if existing_recovery is not None and existing_recovery != recovery:
                raise PackageError("Changed recovery receipt held; preserve evidence.")
            current_desktop, current_state = read_file(self.desktop, missing=True), read_file(self.root / "active.json", missing=True)
            if current_desktop not in (old_desktop, new_desktop) or current_state not in (old_state, new_state):
                raise PackageError("Recovery found unfamiliar changes. No file was overwritten.")
            replace_file(self.desktop, old_desktop, current_desktop)
            replace_file(self.root / "active.json", old_state, current_state)
            if existing_recovery is None:
                write_new(recovery_path, recovery)
            replace_file(self.root / "pending.json", None, raw)
            return {"recovered": True, "versions": "RETAINED"}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    build = commands.add_parser("build", help="Build only the fixed source allowlist")
    build.add_argument("--output", required=True)
    verify = commands.add_parser("verify")
    verify.add_argument("package")
    verify.add_argument("--expected-sha256", required=True)
    install = commands.add_parser("install")
    install.add_argument("package")
    install.add_argument("--expected-sha256", required=True)
    for command in (install, commands.add_parser("activate"), commands.add_parser("remove"), commands.add_parser("recover")):
        command.add_argument("--home", default=str(Path.home()))
    install.add_argument("--python", default="/usr/bin/python3")
    activate = commands.choices["activate"]
    activate.add_argument("package_id")
    activate.add_argument("--python", default="/usr/bin/python3")
    arguments = parser.parse_args(argv)
    try:
        if arguments.command == "build":
            source = Path(__file__).absolute().parents[2]
            revision = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
            dirty = bool(subprocess.check_output(["git", "-C", str(source), "status", "--porcelain", "--", *FILES.values()], text=True))
            result = build_package(source, arguments.output, revision, dirty)
        elif arguments.command in ("verify", "install"):
            manifest, payload = verify_package(arguments.package, arguments.expected_sha256)
            result = {"verified": True, "package_id": manifest["package_id"], "files": len(payload)} if arguments.command == "verify" else Installation(arguments.home).install(manifest, payload, arguments.python)
        elif arguments.command == "activate":
            result = Installation(arguments.home).activate(arguments.package_id, arguments.python)
        elif arguments.command == "remove":
            result = Installation(arguments.home).remove()
        else:
            result = Installation(arguments.home).recover()
        print(json.dumps(result, sort_keys=True))
        return 0
    except (PackageError, OSError, ValueError, KeyError, TypeError, zipfile.BadZipFile, subprocess.CalledProcessError) as error:
        print(f"LFS Pet package held: {error}. No dependencies, browser or service were started.", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
