"""Drives the shakedown command line and watches the processes it starts.

Everything here goes through the public interface: the command line with --json, the files the
contract names (runner.json, the run folder) and the operating system's process list.
"""

import csv
import io
import json
import os
import subprocess
import threading
import time
from pathlib import Path

RUNNER_JSON = Path(os.environ.get("LOCALAPPDATA", "")) / "Shakedown" / "runner.json"


class Answer:
    def __init__(self, args, code, stdout, stderr, seconds):
        self.args, self.code, self.stdout, self.stderr, self.seconds = args, code, stdout, stderr, seconds

    def json(self):
        try:
            return json.loads(self.stdout)
        except ValueError as e:
            raise AssertionError(f"`shakedown {' '.join(self.args)}` printed no JSON document: {e}; "
                                 f"stdout {self.stdout[:300]!r}, stderr {self.stderr[:300]!r}")

    def __str__(self):
        return f"`shakedown {' '.join(self.args)}` exit {self.code} in {self.seconds:.1f} s"


class Shakedown:
    def __init__(self, exe, project, log):
        self.exe, self.project, self.log = str(exe), Path(project), log

    def __call__(self, *args, ceiling=120):
        started = time.monotonic()
        proc = subprocess.run([self.exe, *args], cwd=self.project, capture_output=True, text=True,
                              encoding="utf-8", errors="replace", timeout=ceiling)
        answer = Answer(list(args), proc.returncode, proc.stdout, proc.stderr, time.monotonic() - started)
        self.log(f"{answer}")
        return answer

    def start(self, *args):
        """Starts `shakedown <args>` without waiting for it; returns a Background. The caller ends it."""
        self.log(f"`shakedown {' '.join(args)}` started in the background")
        return Background([self.exe, *args], self.project)

    def run_folders(self):
        runs = self.project / ".shakedown" / "runs"
        return sorted(p.name for p in runs.iterdir()) if runs.is_dir() else []


class Background:
    """A command running in the background whose stdout and stderr are read while it runs.

    A command writes its output as it ends, and the output of a failed start passes the 4 KB a
    Windows pipe holds, so a pipe nobody reads keeps the command from exiting (docs/dedicated-sessions.md).
    """

    def __init__(self, argv, cwd):
        self.proc = subprocess.Popen(argv, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                     text=True, encoding="utf-8", errors="replace")
        self.pid = self.proc.pid
        self.out, self.err = [], []
        self.readers = [threading.Thread(target=lambda s=s, b=b: b.append(s.read()), daemon=True)
                        for s, b in ((self.proc.stdout, self.out), (self.proc.stderr, self.err))]
        for r in self.readers:
            r.start()

    def poll(self):
        return self.proc.poll()

    @property
    def returncode(self):
        return self.proc.returncode

    def communicate(self, timeout):
        """Waits up to `timeout` s for the command to exit; returns (stdout, stderr). Raises TimeoutExpired."""
        self.proc.wait(timeout=timeout)
        for r in self.readers:
            r.join(timeout=30)
        return "".join(self.out), "".join(self.err)


def pids(image):
    """Process ids of every process with this image name."""
    out = subprocess.run(["tasklist", "/FO", "CSV", "/NH", "/FI", f"IMAGENAME eq {image}"],
                         capture_output=True, text=True, timeout=30).stdout
    return {int(row[1]) for row in csv.reader(io.StringIO(out)) if len(row) > 1 and row[0].lower() == image.lower()}


def game_pids():
    """Process ids of every game client, headless client and dedicated server."""
    return pids("arma3_x64.exe") | pids("arma3server_x64.exe")


def alive(pid):
    out = subprocess.run(["tasklist", "/FO", "CSV", "/NH", "/FI", f"PID eq {pid}"],
                         capture_output=True, text=True, timeout=30).stdout
    return any(len(row) > 1 and row[1] == str(pid) for row in csv.reader(io.StringIO(out)))


def kill(pid):
    subprocess.run(["taskkill", "/PID", str(pid), "/T", "/F"], capture_output=True, timeout=30)


def wait_gone(pid, ceiling):
    """Polls once a second, at most `ceiling` times; returns the seconds it took, or None."""
    started = time.monotonic()
    for _ in range(int(ceiling)):
        if not alive(pid):
            return time.monotonic() - started
        time.sleep(1)
    return None if alive(pid) else time.monotonic() - started


def runner_pid():
    """The pid runner.json names, or None when there is no file."""
    try:
        return json.loads(RUNNER_JSON.read_text(encoding="utf-8"))["pid"]
    except (OSError, ValueError, KeyError):
        return None
