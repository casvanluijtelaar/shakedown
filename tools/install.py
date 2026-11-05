"""Builds Shakedown and assembles its install folder (spec/07-tech-stack.md, "Install layout").

    python tools/install.py [destination]

The destination defaults to target/shakedown in the repository. It receives:

    shakedown.exe
    @shakedown/addons/shakedown_main.pbo
    @shakedown/shakedown_x64.dll

Needs cargo and hemtt on PATH. Stops with the failing command's output when a build fails.
"""

import os
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def run(args, cwd):
    print("+", " ".join(args), flush=True)
    done = subprocess.run(args, cwd=cwd)
    if done.returncode != 0:
        sys.exit(f"{args[0]} failed with exit code {done.returncode}")


def main():
    dest = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.path.join(ROOT, "target", "shakedown")
    run(["cargo", "build", "--release", "-p", "shakedown", "-p", "shakedown-ext"], ROOT)
    addon = os.path.join(ROOT, "addon")
    run(["hemtt", "build"], addon)

    exe = "shakedown.exe" if os.name == "nt" else "shakedown"
    dll = "shakedown_ext.dll" if os.name == "nt" else "libshakedown_ext.so"
    ext = "shakedown_x64.dll" if os.name == "nt" else "shakedown_x64.so"
    release = os.path.join(ROOT, "target", "release")
    pbo = os.path.join(addon, ".hemttout", "build", "addons", "shakedown_main.pbo")

    mod = os.path.join(dest, "@shakedown")
    os.makedirs(os.path.join(mod, "addons"), exist_ok=True)
    shutil.copy2(os.path.join(release, exe), os.path.join(dest, exe))
    shutil.copy2(os.path.join(release, dll), os.path.join(mod, ext))
    shutil.copy2(pbo, os.path.join(mod, "addons", "shakedown_main.pbo"))
    print(f"installed to {dest}")


if __name__ == "__main__":
    main()
