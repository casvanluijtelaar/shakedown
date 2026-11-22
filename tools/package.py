"""Builds a release of Shakedown and assembles its zip (DST-01, NFR-LG-04).

    python tools/package.py [--tag v<version>] [--out <folder>] [--no-build]

With --tag it first checks that the tag's version equals the workspace version in Cargo.toml and
stops with exit code 1 when it does not, before it builds anything. It then builds shakedown.exe
and the extension with cargo (--locked) and the addon with HEMTT, writes THIRD-PARTY-LICENSES.txt
for every package compiled into the two binaries, and writes into the output folder (by default
target/dist):

    shakedown-windows-x64.zip           shakedown.exe, @shakedown/, LICENSE, THIRD-PARTY-LICENSES.txt
    shakedown-windows-x64.zip.sha256    "<64 lowercase hex digits>  shakedown-windows-x64.zip"

--no-build packages what an earlier build left in target/release and addon/.hemttout.
Needs cargo, and hemtt unless --no-build is given.
"""

import hashlib
import json
import os
import re
import subprocess
import sys
import tomllib
import zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ZIP_NAME = "shakedown-windows-x64.zip"
TARGET = "x86_64-pc-windows-msvc"
# The workspace crates whose binaries a release ships.
SHIPPED = ("shakedown", "shakedown-ext")
LICENCE_FILE = re.compile(r"^(licen[cs]e|copying|notice|copyright|unlicense)", re.IGNORECASE)
# Packages that ship no licence file of their own, with what the listing says instead. Any other
# package without one stops the packaging, so a new dependency is looked at before it ships.
NO_LICENCE_FILE = {
    "arma-rs": "Declared MIT on crates.io; the package ships no licence file. Its repository, "
    "https://github.com/BrettMayson/arma-rs, carries the GNU GPL version 2 text, which is LICENSE "
    "in this zip.",
    "arma-rs-proc": "Declared MIT on crates.io; the package ships no licence file. It is part of "
    "arma-rs, https://github.com/BrettMayson/arma-rs, whose repository carries the GNU GPL "
    "version 2 text, which is LICENSE in this zip.",
}


def workspace_version(root=ROOT):
    with open(os.path.join(root, "Cargo.toml"), "rb") as f:
        return tomllib.load(f)["workspace"]["package"]["version"]


def check_tag(tag, version):
    """Returns None when the tag names the version, else the reason it does not."""
    if tag == f"v{version}":
        return None
    return f"the tag {tag} does not match the workspace version {version} in Cargo.toml (want v{version})"


def is_prerelease(version):
    return "-" in version


def shipped_packages(metadata):
    """The packages compiled into the shipped binaries: their normal dependencies, transitively."""
    packages = {p["id"]: p for p in metadata["packages"]}
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    members = set(metadata["workspace_members"])
    todo = [i for i in members if packages[i]["name"] in SHIPPED]
    seen = set()
    while todo:
        for dep in nodes[todo.pop()]["deps"]:
            normal = any(kind["kind"] is None for kind in dep["dep_kinds"])
            if normal and dep["pkg"] not in seen:
                seen.add(dep["pkg"])
                todo.append(dep["pkg"])
    found = [packages[i] for i in seen - members]
    return sorted(found, key=lambda p: (p["name"], p["version"]))


def read_text(path):
    with open(path, "rb") as f:
        return f.read().decode("utf-8", errors="replace").replace("\r\n", "\n").rstrip() + "\n"


def third_party_licenses(metadata):
    """The text of THIRD-PARTY-LICENSES.txt; stops when a package has no licence file or note."""
    packages = shipped_packages(metadata)
    missing = []
    parts = [
        "Third-party licences of Shakedown\n\n"
        "shakedown.exe and @shakedown\\shakedown_x64.dll are built from Shakedown's own source, under\n"
        "the licence in LICENSE, and from the packages below. For each package this file holds its\n"
        "declared licence and every licence, copyright and notice file it ships.\n\n"
        + "".join(f"  {p['name']} {p['version']} ({p['license'] or 'no declared licence'})\n" for p in packages)
    ]
    for p in packages:
        folder = os.path.dirname(p["manifest_path"])
        files = sorted(f for f in os.listdir(folder) if LICENCE_FILE.match(f) and os.path.isfile(os.path.join(folder, f)))
        head = f"{'=' * 78}\n{p['name']} {p['version']}\nLicence: {p['license'] or 'none declared'}\n"
        if p.get("repository"):
            head += f"Source: {p['repository']}\n"
        if not files:
            if p["name"] not in NO_LICENCE_FILE:
                missing.append(f"{p['name']} {p['version']}")
                continue
            parts.append(head + "\n" + NO_LICENCE_FILE[p["name"]] + "\n")
            continue
        body = "".join(f"\n--- {f} ---\n\n{read_text(os.path.join(folder, f))}" for f in files)
        parts.append(head + body)
    if missing:
        raise SystemExit(
            "these packages ship no licence file and have no note in tools/package.py: " + ", ".join(missing)
        )
    return "\n".join(parts)


def cargo_metadata(root=ROOT):
    done = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked", "--filter-platform", TARGET],
        cwd=root, capture_output=True, text=True, encoding="utf-8",
    )
    if done.returncode != 0:
        raise SystemExit(f"cargo metadata failed: {done.stderr}")
    return json.loads(done.stdout)


def write_zip(path, entries):
    """entries: (name in the zip, file on disk or bytes) pairs. Names use forward slashes."""
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for name, source in entries:
            if isinstance(source, bytes):
                z.writestr(name, source)
            else:
                z.write(source, name)


def sha256_line(path):
    digest = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            digest.update(block)
    return f"{digest.hexdigest()}  {os.path.basename(path)}\n"


def run(args, cwd):
    print("+", " ".join(args), flush=True)
    if subprocess.run(args, cwd=cwd).returncode != 0:
        raise SystemExit(f"{args[0]} failed")


def main(argv):
    tag = out = None
    build = True
    while argv:
        arg = argv.pop(0)
        if arg == "--tag":
            tag = argv.pop(0)
        elif arg == "--out":
            out = os.path.abspath(argv.pop(0))
        elif arg == "--no-build":
            build = False
        else:
            raise SystemExit(f"unknown argument {arg}")
    version = workspace_version()
    if tag is not None:
        reason = check_tag(tag, version)
        if reason:
            raise SystemExit(reason)
    out = out or os.path.join(ROOT, "target", "dist")
    if build:
        run(["cargo", "build", "--release", "--locked", "-p", "shakedown", "-p", "shakedown-ext"], ROOT)
        run(["hemtt", "build"], os.path.join(ROOT, "addon"))
    release = os.path.join(ROOT, "target", "release")
    licences = third_party_licenses(cargo_metadata()).encode("utf-8")
    os.makedirs(out, exist_ok=True)
    zip_path = os.path.join(out, ZIP_NAME)
    write_zip(zip_path, [
        ("shakedown.exe", os.path.join(release, "shakedown.exe")),
        ("@shakedown/shakedown_x64.dll", os.path.join(release, "shakedown_ext.dll")),
        ("@shakedown/addons/shakedown_main.pbo",
         os.path.join(ROOT, "addon", ".hemttout", "build", "addons", "shakedown_main.pbo")),
        ("LICENSE", os.path.join(ROOT, "LICENSE")),
        ("THIRD-PARTY-LICENSES.txt", licences),
    ])
    with open(zip_path + ".sha256", "w", encoding="ascii", newline="\n") as f:
        f.write(sha256_line(zip_path))
    print(f"version {version}{' (pre-release)' if is_prerelease(version) else ''}: {zip_path}")


if __name__ == "__main__":
    main(sys.argv[1:])
