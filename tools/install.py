"""Builds Shakedown and assembles its install folder (spec/07-tech-stack.md, "Install layout").

    python tools/install.py [destination] [--game <Arma 3 folder>]

The destination defaults to target/shakedown in the repository. It receives:

    shakedown.exe
    @shakedown/addons/shakedown_main.pbo
    @shakedown/shakedown_x64.dll
    fixtures/@skd_fixture/addons/skdfx_main.pbo
    fixtures/@skd_fixture/skdfx_x64.dll     the fixture's extension (FIX-01)
    fixtures/@skd_fixture_cba/addons/skdfx_cba_main.pbo
    fixtures/skdfx/addons/main/...        the fixture's unpacked file tree
    fixtures/skdfx_cba/addons/main/...    the CBA fixture's unpacked file tree

On Windows it then links, in the game folder, <game>\\skdfx and <game>\\skdfx_cba to the unpacked
trees and <game>\\@skd_fixture to the installed @skd_fixture, as spec/contract/fixture-mod.md
says. A link left by an earlier install is replaced; a real folder of that name is never touched.
The game folder defaults to the SHAKEDOWN_GAME variable, else Steam's default Arma 3 folder.

Needs cargo and hemtt on PATH. Stops with the failing command's output when a build fails.
"""

import os
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_GAME = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3"

# (source folder in fixtures/, installed mod folder, game prefix of its addons)
FIXTURES = [
    ("skd_fixture", "@skd_fixture", "skdfx"),
    ("skd_fixture_cba", "@skd_fixture_cba", "skdfx_cba"),
]


def run(args, cwd):
    print("+", " ".join(args), flush=True)
    done = subprocess.run(args, cwd=cwd)
    if done.returncode != 0:
        sys.exit(f"{args[0]} failed with exit code {done.returncode}")


def copy_tree(src, dst):
    if os.path.isdir(dst):
        shutil.rmtree(dst)
    shutil.copytree(src, dst)


def is_link(path):
    return os.path.islink(path) or (hasattr(os.path, "isjunction") and os.path.isjunction(path))


def link(game, name, target):
    """Points <game>\\<name> at target with a directory junction."""
    path = os.path.join(game, name)
    if is_link(path):
        os.rmdir(path) if not os.path.islink(path) else os.unlink(path)
    elif os.path.exists(path):
        sys.exit(f"{path} exists and is not a link; move it away and install again")
    done = subprocess.run(["cmd", "/c", "mklink", "/J", path, target], capture_output=True, text=True)
    if done.returncode != 0:
        sys.exit(f"cannot link {path} to {target}: {done.stdout}{done.stderr}")
    print(f"linked {path} -> {target}")


def main():
    args = sys.argv[1:]
    game = os.environ.get("SHAKEDOWN_GAME", DEFAULT_GAME)
    if "--game" in args:
        i = args.index("--game")
        game = args[i + 1]
        del args[i : i + 2]
    dest = os.path.abspath(args[0]) if args else os.path.join(ROOT, "target", "shakedown")
    run(["cargo", "build", "--release", "-p", "shakedown", "-p", "shakedown-ext", "-p", "skdfx-ext"], ROOT)
    addon = os.path.join(ROOT, "addon")
    run(["hemtt", "build"], addon)
    for folder, _, _ in FIXTURES:
        run(["hemtt", "build"], os.path.join(ROOT, "fixtures", folder))

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

    installed = os.path.join(dest, "fixtures")
    for folder, mod_name, prefix in FIXTURES:
        source = os.path.join(ROOT, "fixtures", folder)
        copy_tree(os.path.join(source, ".hemttout", "build", "addons"), os.path.join(installed, mod_name, "addons"))
        copy_tree(os.path.join(source, "addons"), os.path.join(installed, prefix, "addons"))
    fixture_dll = "skdfx_ext.dll" if os.name == "nt" else "libskdfx_ext.so"
    fixture_ext = "skdfx_x64.dll" if os.name == "nt" else "skdfx_x64.so"
    shutil.copy2(os.path.join(release, fixture_dll), os.path.join(installed, "@skd_fixture", fixture_ext))

    if os.name == "nt":
        if not os.path.isdir(game):
            sys.exit(f"the game folder {game} does not exist; give --game")
        for _, _, prefix in FIXTURES:
            link(game, prefix, os.path.join(installed, prefix))
        link(game, "@skd_fixture", os.path.join(installed, "@skd_fixture"))
    print(f"installed to {dest}")


if __name__ == "__main__":
    main()
