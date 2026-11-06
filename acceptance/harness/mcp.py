"""A small MCP client over stdio, as Claude Code is one: it starts the server the project's .mcp.json
declares, speaks newline-delimited JSON-RPC 2.0 to it, and keeps every message the server sends.

Written here instead of taking an MCP SDK as a dependency; it knows the few messages the checks use.
"""

import json
import os
import shutil
import subprocess
import threading
import time
from pathlib import Path

PROTOCOL = "2025-06-18"


class Mcp:
    def __init__(self, project, install, log):
        """Starts the server named `shakedown` in the project's .mcp.json, with the install folder on PATH
        (docs/agent-tools.md: `shakedown` must be on PATH), in the project folder."""
        self.log = log
        server = json.loads((Path(project) / ".mcp.json").read_text(encoding="utf-8"))["mcpServers"]["shakedown"]
        env = dict(os.environ)
        env["PATH"] = str(install) + os.pathsep + env.get("PATH", "")
        command = shutil.which(server["command"], path=env["PATH"])
        assert command, f".mcp.json names the command {server['command']!r}, which is not on PATH with the install folder"
        self.timeout_s = server.get("timeout", 60000) / 1000
        self.proc = subprocess.Popen([command, *server.get("args", [])], cwd=project, env=env,
                                     stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.pid = self.proc.pid
        self.lock = threading.Condition()
        self.responses = {}      # id -> (monotonic time, message)
        self.notes = []          # (monotonic time, message) of every notification and request from the server
        self.stderr = []
        self.next_id = 0
        threading.Thread(target=self._read, daemon=True).start()
        threading.Thread(target=lambda: self.stderr.append(self.proc.stderr.read().decode("utf-8", "replace")),
                         daemon=True).start()
        log(f"`shakedown mcp` started from .mcp.json, pid {self.pid}")

    def _read(self):
        for raw in self.proc.stdout:
            now = time.monotonic()
            try:
                msg = json.loads(raw.decode("utf-8"))
            except ValueError:
                with self.lock:
                    self.notes.append((now, {"unparsed": raw.decode("utf-8", "replace")}))
                continue
            if "method" in msg and "id" in msg:  # a request from the server: answer ping, refuse the rest
                reply = {"jsonrpc": "2.0", "id": msg["id"], "result": {}} if msg["method"] == "ping" else \
                    {"jsonrpc": "2.0", "id": msg["id"], "error": {"code": -32601, "message": "not supported"}}
                self._write(reply)
            with self.lock:
                if "method" in msg:
                    self.notes.append((now, msg))
                else:
                    self.responses[msg.get("id")] = (now, msg)
                self.lock.notify_all()
        with self.lock:
            self.lock.notify_all()

    def _write(self, msg):
        self.proc.stdin.write((json.dumps(msg) + "\n").encode("utf-8"))
        self.proc.stdin.flush()

    def send(self, method, params=None):
        """Sends a request and returns its id at once."""
        with self.lock:
            self.next_id += 1
            rid = self.next_id
        msg = {"jsonrpc": "2.0", "id": rid, "method": method}
        if params is not None:
            msg["params"] = params
        self._write(msg)
        return rid

    def notify(self, method, params=None):
        msg = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            msg["params"] = params
        self._write(msg)

    def wait(self, rid, ceiling):
        """The response to request rid, waiting at most `ceiling` s; raises AssertionError without one."""
        end = time.monotonic() + ceiling
        with self.lock:
            while rid not in self.responses:
                left = end - time.monotonic()
                if left <= 0 or self.proc.poll() is not None:
                    break
                self.lock.wait(min(left, 1))
            if rid not in self.responses:
                raise AssertionError(f"the MCP server gave no response to request {rid} within {ceiling} s"
                                     f" (server {'exited ' + str(self.proc.poll()) if self.proc.poll() is not None else 'running'})")
            return self.responses[rid]

    def request(self, method, params=None, ceiling=70):
        rid = self.send(method, params)
        _, msg = self.wait(rid, ceiling)
        assert "error" not in msg, f"{method} answered a JSON-RPC error: {msg['error']}"
        return msg["result"]

    def initialize(self):
        result = self.request("initialize", {"protocolVersion": PROTOCOL, "capabilities": {},
                                             "clientInfo": {"name": "shakedown-acceptance", "version": "0"}}, 30)
        self.notify("notifications/initialized")
        self.info = result
        return result

    def tools(self):
        tools, cursor = [], None
        while True:
            result = self.request("tools/list", {"cursor": cursor} if cursor else {}, 30)
            tools += result["tools"]
            cursor = result.get("nextCursor")
            if not cursor:
                return tools

    def start_call(self, name, arguments, token=None):
        """Starts tools/call and returns (request id, monotonic start time)."""
        params = {"name": name, "arguments": arguments}
        if token is not None:
            params["_meta"] = {"progressToken": token}
        started = time.monotonic()
        return self.send("tools/call", params), started

    def call(self, name, arguments=None, token=None, ceiling=70):
        """tools/call; returns (result, seconds). A JSON-RPC error fails with its message."""
        rid, started = self.start_call(name, arguments or {}, token)
        at, msg = self.wait(rid, ceiling)
        self.log(f"mcp {name} {json.dumps(arguments or {})[:120]} answered in {at - started:.2f} s"
                 f"{' isError' if msg.get('result', {}).get('isError') else ''}")
        assert "error" not in msg, f"tools/call {name} answered a JSON-RPC error: {msg['error']}"
        return msg["result"], at - started

    def cancel(self, rid, reason="cancelled by the acceptance check"):
        self.notify("notifications/cancelled", {"requestId": rid, "reason": reason})

    def progress(self, token):
        """(monotonic time, params) of every progress notification for the token."""
        with self.lock:
            return [(t, m["params"]) for t, m in self.notes
                    if m.get("method") == "notifications/progress" and m.get("params", {}).get("progressToken") == token]

    def close(self):
        """Closes stdin, waits up to 10 s for the server to exit, then stops it by its id."""
        try:
            self.proc.stdin.close()
        except OSError:
            pass
        try:
            self.proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.log(f"`shakedown mcp` {self.pid} did not exit 10 s after its stdin closed; stopping it by id")
            subprocess.run(["taskkill", "/PID", str(self.pid), "/T", "/F"], capture_output=True, timeout=30)
            self.proc.wait(timeout=30)

    def __enter__(self):
        self.initialize()
        return self

    def __exit__(self, *exc):
        self.close()


def text_of(result):
    content = result.get("content") or []
    assert len(content) == 1 and content[0].get("type") == "text", \
        f"the result holds {len(content)} content blocks, expected one text block: {str(content)[:300]}"
    return content[0]["text"]


def structured(result):
    """The structuredContent of a successful call, which its one text block must repeat (AGT-02)."""
    assert not result.get("isError"), f"the call returned isError: {str(result.get('content'))[:500]}"
    assert "structuredContent" in result, f"the result has no structuredContent: {str(result)[:300]}"
    text = text_of(result)
    try:
        same = json.loads(text)
    except ValueError:
        raise AssertionError(f"the text block is not JSON: {text[:300]!r}")
    assert same == result["structuredContent"], \
        f"the text block's JSON differs from structuredContent: {text[:300]!r} vs {str(result['structuredContent'])[:300]}"
    return result["structuredContent"]


def tool_error(result, code):
    """The text of an isError result with this code, of the form `<code>: <what is wrong>. <what to do next>.`"""
    assert result.get("isError") is True, f"expected isError with {code}, got {str(result)[:500]}"
    text = text_of(result)
    assert text.startswith(code + ": "), f"the error text starts {text[:80]!r}, expected {code}: "
    rest = text[len(code) + 2:].strip()
    assert rest.endswith("."), f"the error text does not end in a full stop: {text!r}"
    assert rest.count(". ") >= 1 or rest.count(".\n") >= 1, \
        f"the error text is not of the form <what is wrong>. <what to do next>.: {text!r}"
    return text
