"""Runs one command in this process's own console and sends it a console event when told to.

Started by m08.py with CREATE_NEW_CONSOLE, so the console is one of its own and no other process of
the machine shares it:

    python console_signal.py <mode> <go-file> <state-file> <stdout-file> <stderr-file> <exe> <args...>

mode `c` sends CTRL_C_EVENT to the whole console (group 0), as a person pressing Ctrl+C in it does;
this process ignores the event itself. mode `break` starts the command in a process group of its own
and sends CTRL_BREAK_EVENT to that group, as cli.md says a program that drives the command line does.

It starts the command, writes `{"pid": ...}` to the state file, waits for the go file (at most 600 s),
sends the event, waits for the command to end (at most 600 s) and writes `{"pid", "code", "sent_at"}`.
"""

import ctypes
import json
import subprocess
import sys
import time

CTRL_C_EVENT, CTRL_BREAK_EVENT = 0, 1
CREATE_NEW_PROCESS_GROUP = 0x00000200


def main():
    mode, go, state, out, err, *argv = sys.argv[1:]
    k32 = ctypes.windll.kernel32
    flags = CREATE_NEW_PROCESS_GROUP if mode == "break" else 0
    # A process may inherit the ignoring of Ctrl+C from whatever started it; a console a person opens
    # does not ignore it, so the command starts with Ctrl+C handled, as it would there.
    k32.SetConsoleCtrlHandler(None, False)
    with open(out, "wb") as o, open(err, "wb") as e:
        proc = subprocess.Popen(argv, stdout=o, stderr=e, stdin=subprocess.DEVNULL, creationflags=flags)
        # Set after the start, so the command does not inherit the ignoring of Ctrl+C.
        k32.SetConsoleCtrlHandler(None, True)
        with open(state, "w") as s:
            json.dump({"pid": proc.pid}, s)
        sent_at = None
        for _ in range(1200):
            if proc.poll() is not None:
                break
            try:
                open(go).close()
            except OSError:
                time.sleep(0.5)
                continue
            sent_at = time.time()
            ok = (k32.GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0) if mode == "c"
                  else k32.GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, proc.pid))
            if not ok:
                sent_at = f"GenerateConsoleCtrlEvent failed: {k32.GetLastError()}"
            break
        try:
            code = proc.wait(timeout=600)
        except subprocess.TimeoutExpired:
            proc.kill()
            code = proc.wait(timeout=30)
    with open(state, "w") as s:
        json.dump({"pid": proc.pid, "code": code, "sent_at": sent_at}, s)


if __name__ == "__main__":
    main()
