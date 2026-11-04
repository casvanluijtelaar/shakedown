---
type: Research
title: "Agent tools: MCP"
description: What the current Model Context Protocol, Claude Code's MCP loading, Playwright's MCP server and the Arma MCP servers say about how the framework exposes its capabilities to an agent, and how to run that server safely on the local machine.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T21:55:00+01:00 }
sources:
  - { id: mcp-spec, resource: "https://modelcontextprotocol.io/specification/latest (resolves to revision 2026-07-28)", read: 2026-11-04 }
  - { id: mcp-changelog, resource: "https://modelcontextprotocol.io/specification/2026-07-28/changelog.md", read: 2026-11-04 }
  - { id: mcp-transports, resource: "https://modelcontextprotocol.io/specification/2026-07-28/basic/transports.md", read: 2026-11-04 }
  - { id: mcp-stdio, resource: "https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio.md", read: 2026-11-04 }
  - { id: mcp-http, resource: "https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http.md", read: 2026-11-04 }
  - { id: mcp-progress, resource: "https://modelcontextprotocol.io/specification/2026-07-28/basic/utilities/progress.md", read: 2026-11-04 }
  - { id: mcp-cancel, resource: "https://modelcontextprotocol.io/specification/2026-07-28/basic/patterns/cancellation.md", read: 2026-11-04 }
  - { id: mcp-tools, resource: "https://modelcontextprotocol.io/specification/2026-07-28/server/tools.md", read: 2026-11-04 }
  - { id: mcp-tasks, resource: "https://modelcontextprotocol.io/extensions/tasks/overview.md", read: 2026-11-04 }
  - { id: mcp-auth, resource: "https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization.md", read: 2026-11-04 }
  - { id: mcp-security, resource: "https://modelcontextprotocol.io/specification/2026-07-28/basic/security_best_practices.md", read: 2026-11-04 }
  - { id: cc-mcp, resource: "https://code.claude.com/docs/en/mcp.md", read: 2026-11-04 }
  - { id: pw-mcp, resource: "https://github.com/microsoft/playwright-mcp README at commit 50b004fb3d3ee17f9710f067ed1b6d59fd16e4a1 (release v0.0.84)", read: 2026-11-04 }
  - { id: mae-mcp, resource: "https://docs.maestro.dev/get-started/maestro-mcp.md", read: 2026-11-04 }
  - { id: headswe, resource: "https://github.com/headswe/arma3-mcp README at commit d778d75e87bf7590793297e1afb63558c8b88f14 (no license file per the GitHub API)", read: 2026-11-04 }
  - { id: neefay, resource: "https://github.com/Neefay/arma3-llm-harness README at commit 21aac67e8184901e4016ed4effa06c99afd2e067 (no license file per the GitHub API)", read: 2026-11-04 }
  - { id: zihuange, resource: "https://github.com/Zihuange/arma3-mcp README, adapters/*.py and docs/safety.md at commit 380af427af8604f1d04996946c6e3ccfa325ed10 (MIT)", read: 2026-11-04 }
  - { id: a3bridge, resource: "https://github.com/lepitinox/a3-bridge README at commit cf42c73914052405238b5646d1449b406eef887d (MIT)", read: 2026-11-04 }
  - { id: a3ref, resource: "https://github.com/a3ref/mcp README at commit 3109b0ef97d2e282b142c3ab0e6fd811f897a93f (MIT)", read: 2026-11-04 }
  - { id: wikimcp, resource: "https://github.com/ScottTMConnors/arma3-wiki-mcp README at commit 90418253ccd85a91d94c570f5525f4fe0d3c14f6", read: 2026-11-04 }
---

# Agent tools: MCP

This topic answers how the framework hands its capabilities to an AI agent as tools (`CR-53`), and how the local control channel stays safe (`CR-60`). It also touches `CR-27` (ask a running game what is going on), `CR-20` (waits, which become long tool calls) and `CR-05` (who owns the game processes). Licenses of the Arma tools are T11's; this file uses them only to say whether code may be taken.

## The protocol at its current version

The specification at `modelcontextprotocol.io/specification/latest` is revision **2026-07-28**, defined by its TypeScript schema[^mcp-spec]. It uses JSON-RPC 2.0 between hosts, clients and servers; servers offer resources, prompts and tools, and tasks are an optional extension[^mcp-spec]. The spec says tools "represent arbitrary code execution and must be treated with appropriate caution", that hosts must get user consent before invoking a tool, and that tool annotations are untrusted unless the server is trusted[^mcp-spec].

### What changed since 2025-11-25

The revision is a break from the one before it[^mcp-changelog]:
- MCP is stateless. The `initialize` handshake is gone; every request carries its protocol version and client capabilities in `_meta`, and servers must implement `server/discover` to advertise versions and capabilities.
- Streamable HTTP has no protocol-level sessions and no `Mcp-Session-Id`. "Servers that need cross-call state use explicit, server-minted handles passed as ordinary tool arguments."
- SSE resumability is removed: "A broken response stream loses the in-flight request"; the client re-issues it.
- Servers no longer send requests to clients. Sampling, elicitation and roots go through Multi Round-Trip Requests: the server returns an `InputRequiredResult` and the client retries with the answers. Roots, Sampling and Logging are deprecated; the suggested replacement for Logging is `stderr` on stdio.
- Tasks moved out of the core into the `io.modelcontextprotocol/tasks` extension.
- `inputSchema` and `outputSchema` accept any JSON Schema 2020-12 keyword; `structuredContent` can be any JSON value. Servers should list tools in a deterministic order.

**Forces:** the framework does not hand-roll the protocol. It uses an MCP SDK that speaks both 2026-07-28 and the earlier, initialization-based revisions, because clients in use today still speak the earlier one (see Claude Code below). Which SDK and version is a spec and tech-stack choice, looked up at that time. Whether the official SDKs already support 2026-07-28 is **unverified**. Session state that spans tool calls (a running session, a run) is carried as a handle the server mints and the agent passes back, not as protocol session state. Touches `CR-53`, `CR-01`.

### Transports

Two standard transports exist[^mcp-transports]:
- **stdio**: the client launches the server as a subprocess; one JSON-RPC message per line on stdin and stdout, nothing else on stdout, logs on stderr[^mcp-stdio]. Shutdown is the client closing stdin, waiting, then killing the process; on Windows "clients can use `TerminateProcess` or Job Objects". If the server dies, the client should restart it, and in-flight requests are lost[^mcp-stdio].
- **Streamable HTTP**: each message is a POST to one endpoint, the answer is JSON or a request-scoped SSE stream[^mcp-transports]. Servers "MUST validate the `Origin` header on all incoming connections to prevent DNS rebinding attacks", "SHOULD bind only to localhost (127.0.0.1)" when local, and "SHOULD implement proper authentication for all connections"[^mcp-http].

The old HTTP+SSE transport is deprecated[^mcp-changelog]. Custom transports over a byte stream such as TCP should reuse the stdio framing[^mcp-transports].

### Long-running calls, progress and cancellation

A client that wants progress puts a `progressToken` in a request's `_meta`; the server may then send `notifications/progress` with the token, a `progress` value that must increase, an optional `total` and an optional `message`[^mcp-progress]. Request-scoped notifications flow on the response stream of the request they belong to[^mcp-changelog].

To cancel, a stdio client sends `notifications/cancelled` with the request ID; on Streamable HTTP closing the response stream is the cancellation. Servers should stop work and send nothing more for it[^mcp-stdio][^mcp-cancel]. Implementations "SHOULD establish timeouts for all sent requests", may reset the timer on progress, but "SHOULD always enforce a maximum timeout, regardless of progress notifications"[^mcp-cancel].

The tasks extension lets a server answer a long request with a durable task handle (`resultType: "task"`) with a status of `working`, `input_required`, `completed`, `failed` or `cancelled`; the client polls `tasks/get` and can `tasks/cancel`, and the handle survives a reconnect[^mcp-tasks]. Its rationale: "Many clients and transport intermediaries impose timeouts that make this impractical beyond a few seconds"[^mcp-tasks].

**Forces:**
- A wait in a check (`CR-20`) and a whole suite run can both outlast a client's tool timeout. A tool that waits sends progress, and the framework still bounds every call with its own ceiling. Touches `CR-20`, `CR-53`.
- A suite run does not live inside one tool call. A run tool returns a run handle at once, and the agent polls or waits on it with short calls. The tasks extension does this in the protocol, but client support is narrow (see below), so the handle plus a status tool works everywhere and tasks can come later. Touches `CR-53`, `CR-40`, `CR-51`.
- Cancelling a tool call has to stop the game-side work it started (a polling wait, a step), or the session is left in an unknown state for the next check. Touches `CR-02`, `CR-05`.

### Error reporting

Tools report errors two ways[^mcp-tools]:
1. **Protocol errors**, as JSON-RPC errors, for a malformed request, an unknown tool or a server fault. Models are "less likely to be able to fix" them.
2. **Tool execution errors**, in the result with `isError: true`, for input validation and business logic errors, "actionable feedback that language models can use to self-correct and retry". Clients should pass them to the model.

A call against an expired or unknown handle "should return a tool execution error that says so, so the model can recover"[^mcp-tools]. A tool may declare an `outputSchema` and return `structuredContent`, and should also return the same JSON as text for older clients[^mcp-tools].

**Forces:** a failed check is not a tool error. A run tool succeeds and returns the verdict (pass, fail or invalid with the reason) as structured content. `isError: true` is for what the agent can fix: bad SQF that does not compile, an unknown machine name, a session that is not running, an expired handle. Each such error says what to do next. Every tool has an `outputSchema`, so the agent and the result file share one shape. Touches `CR-31`, `CR-51`, `CR-53`.

### Authorization for a local server

The authorization spec (OAuth-based) applies to HTTP transports; "Implementations using an STDIO transport SHOULD NOT follow this specification, and instead retrieve credentials from the environment"[^mcp-auth]. The security best practices list the attacks on local servers: a malicious start command in a client configuration, a malicious payload in the server, and "an insecure local server that's left running on localhost via DNS rebinding"[^mcp-security]. Servers meant to run locally "SHOULD implement measures to prevent unauthorized usage from malicious processes: Use the `stdio` transport to limit access to just the MCP client", or, over HTTP, "Require an authorization token" or use IPC with restricted access[^mcp-security].

**Forces:** the agent-facing MCP server uses stdio, so only the client that launched it can talk to it, and it needs no OAuth. Any socket the framework opens beyond that (to the game's extension, or to a runner process) binds to 127.0.0.1, rejects any request with an `Origin` header or a non-protocol first line, and requires a random token per run, passed through the environment or a file only the user can read. This is `CR-60`. Touches `CR-60`, `CR-61`.

## How Claude Code loads a project MCP server

A project server is declared in `.mcp.json` at the project root under `mcpServers`, meant to be checked in so everyone gets the same tools[^cc-mcp]. An entry with no `type` is read as stdio; an entry with a `url` needs `"type": "http"` (alias `streamable-http`), `"sse"` or `"ws"`[^cc-mcp]. `command`, `args`, `env`, `url` and `headers` expand `${VAR}` and `${VAR:-default}`[^cc-mcp].

Approval: in interactive sessions Claude Code asks for approval before using servers from `.mcp.json`; a server shows `Pending approval` until then. "A cloned repository can't approve its own servers": approvals committed to the project's `.claude/settings.json` are ignored until the user trusts the workspace. In `claude -p` runs and Agent SDK sessions Claude Code cannot show the prompt and loads project servers without asking; `disabledMcpjsonServers` or `--strict-mcp-config` keep a server out[^cc-mcp].

Limits that shape the tools[^cc-mcp]:
- A per-server `timeout` in `.mcp.json` (milliseconds) is a hard wall-clock limit per tool call, and progress does not extend it. Without it, `MCP_TOOL_TIMEOUT` applies, about 28 hours when unset.
- An idle timeout aborts a call that sends no response and no progress for the idle window: 30 minutes for stdio servers, 5 minutes for HTTP. Progress notifications keep a call alive against it.
- A main-conversation call still running after two minutes moves to a background task; the result arrives later as a notification.
- Output above 10,000 tokens warns, and output is cut at 25,000 tokens by default (`MAX_MCP_OUTPUT_TOKENS`). Text results over 50,000 characters are saved to a file unless the tool raises its limit with `_meta["anthropic/maxResultSizeChars"]`, up to 500,000.
- MCP tasks need a server on revision 2026-07-28, a session that fetches feature flags, a terminal CLI or Agent SDK session, and a call from the main conversation, "not from a subagent".
- On its v2 runtime Claude Code asks HTTP and stdio servers whether they support the newer revision and uses it with those that do; it connects to every other server as before.
- Stopping a stdio server: "On Windows: Claude Code closes the server's stdin, then ends the server's process tree if the process is still running 300 milliseconds later."

**Forces:**
- The framework ships a `.mcp.json` entry for a stdio server, and its docs tell the owner to approve it once. On the Windows test machine, agent sessions use it; on the Mac there is no game, so the server must start and answer `status` without one, saying the game is not available, instead of failing to connect. Touches `CR-53`.
- **The game must not be a child of the MCP server.** Claude Code ends the server's process tree on Windows when a session ends, which would kill a session meant to serve many checks (`CR-01`) and an owner's manual session (`CR-52`). The session belongs to a separate, long-lived runner process that owns the game processes and their job object (`CR-05`); the MCP server and the command line are thin clients of it. Neefay's harness reached the same answer: its game processes "are started through WMI, outside the MCP client's job object" and survive MCP server restarts[^neefay]. How to start processes outside the client's job object is T8's question. Touches `CR-01`, `CR-05`, `CR-52`.
- Tool calls stay short. A wait tool takes a ceiling and returns at it; a suite run returns a handle; long calls send progress. Short calls also matter for clients with tighter limits: Neefay notes Codex cancels tool calls at 60 seconds by default and keeps its own listen call at 25 seconds[^neefay]. Touches `CR-20`, `CR-53`.
- Results are compact: a verdict, the step that failed, the last observed value, and paths to the result file and logs, not the logs themselves. Large evidence goes to files the result names. Touches `CR-51`, `CR-24`.
- The server declares tool timeouts in `.mcp.json` (`timeout`) that match the framework's own ceilings, so the client never cuts a call the framework would have ended cleanly. Touches `CR-20`.

## How Playwright's MCP server shapes its tools

Playwright MCP's tools share a `browser_` prefix and are grouped: core automation, tabs, and opt-in groups turned on with `--caps` (network, storage, devtools, vision, pdf, testing)[^pw-mcp]. Each tool lists its parameters and whether it is read-only[^pw-mcp]. The model works from `browser_snapshot`, an accessibility snapshot whose element references (`target`) the action tools take; `browser_find` searches the snapshot instead of returning all of it, "which is cheaper"[^pw-mcp]. `browser_wait_for` waits for text to appear or disappear, with a `time` of "at most 30" seconds[^pw-mcp]. Running arbitrary code is a separate tool named `browser_run_code_unsafe`, described as "RCE-equivalent"; `browser_evaluate` evaluates JavaScript on the page[^pw-mcp]. A `testing` group adds `browser_verify_element_visible`, `browser_verify_text_visible`, `browser_verify_value` and `browser_generate_locator`[^pw-mcp]. Over HTTP it binds to localhost by default and checks hosts with `--allowed-hosts`; its README says it "is **not** a security boundary"[^pw-mcp]. The README also says coding agents "increasingly favor CLI–based workflows exposed as SKILLs over MCP because CLI invocations are more token-efficient"[^pw-mcp]. Maestro's MCP server is much smaller: list devices, inspect screen, screenshot, run flows, a cheat sheet, and cloud tools[^mae-mcp].

**Forces:**
- One prefix, a small core set, more behind opt-in groups, and a read-only flag on each tool. A read-only tool (inspect a machine, read logs, list checks) can be allowed without asking; tools that change the game are not. Touches `CR-53`, `CR-27`.
- A snapshot of state, with a search tool so the agent need not pull everything, is the shape of `CR-27`: "what is around player 1" returns a compact summary, and a query narrows it.
- A cap on wait time per call, like Playwright's 30 seconds.
- Arbitrary SQF is a named, plainly labelled tool, separate from the structured ones. Touches `CR-10`, `CR-60`.
- Assertion tools for the agent, so an agent can try an assertion live before writing it into a check. Touches `CR-50`.
- Every tool has a command-line twin, so an agent without MCP, or one saving tokens, uses the CLI. Touches `CR-53`, `CR-40`.

## How the Arma MCP servers shape theirs

| Server | Shape | Transport and safety | Code reuse |
|---|---|---|---|
| `headswe/arma3-mcp` | ~24 tools: `execute_sqf` (with `args`, `scheduled`, `timeout`, `target` of server, owner id or player), `game_status`, `mission_info`, `list_players`, `list_units`, `get_object`, `create_object`, `teleport`, `set_environment`, `read_rpt`, `performance`, plus `eden_*` editor tools[^headswe] | stdio to the client; the server talks to the game's extension over TCP on 127.0.0.1:9120 with an optional token. The line protocol answers anything other than its commands with an error and closes, "so a web page that POSTs to the port cannot slip an `EXEC` in"[^headswe] | No license file: ideas only |
| `Neefay/arma3-llm-harness` | `arma_start`, `arma_stop`, `arma_status`, `arma_exec(code, target, args, scheduled)` with per-machine results and attributed errors, `arma_events(since, ...)` as a cursor stream, `arma_wait_for(match, since, timeout_s)`, `arma_errors`, `arma_rpt`, `arma_snapshot`, `arma_reload`, `arma_run_scenario`, `arma_guide`[^neefay] | stdio; loopback HTTP to the game with "a random per-run token checked in constant time"; the in-game probe stays inert unless the harness wrote a session file for that game's PID; every request logged to `audit.jsonl`[^neefay] | No license file: ideas only |
| `Zihuange/arma3-mcp` | `exec`, `exec_batch`, `game_events`, `game_link`, `gates_status`, `mission_snapshot`, `player_state`, `scene_snapshot`, `screenshot`, `input_key`, `input_click` and other input tools[^zihuange] | stdio by default; its HTTP mode binds 0.0.0.0 by default for WSL. Commands are graded and gated, with an `unrestricted` profile that turns every gate off and is flagged in `gates_status` and in every audit record[^zihuange] | MIT |
| `lepitinox/a3-bridge` | Every game order as a tool, plus situation, events and waits on chat or events[^a3bridge] | HTTP and WebSocket on 127.0.0.1:8765, rejects requests from other web sites, but "any local program can drive the game, including `exec_sqf`"[^a3bridge] | MIT |
| `a3ref/mcp` | `search`, `get`, `list_group`, `lint_sqf`, `recent_changes`: reference and linter, no game[^a3ref] | stdio | MIT; content belongs to the wiki's authors |
| `ScottTMConnors/arma3-wiki-mcp` | `lookup_command`, `lookup_event`, `search`, `list_group`, tutorials, `refresh_wiki`[^wikimcp] | stdio | Server MIT; wiki data non-commercial only[^wikimcp] |

What they agree on:
- Run SQF on a named machine and get JSON back is the core tool. Arguments travel as data (`args` arrives as `_this`) instead of being formatted into code[^headswe]. Results say which machine answered, and script errors are tied to the request that caused them[^neefay]. Touches `CR-10`, `CR-22`, `CR-23`.
- headswe's warning: "An `ok` result only confirms the SQF request completed; it does not prove the intended in-game effect occurred"[^headswe]. That is why checks wait and assert, and why the agent gets wait and expect tools, not only exec. Touches `CR-20`, `CR-31`.
- Events as a cursor stream (`since`) and a wait tool with a match and a timeout let an agent wait without polling by hand and lose nothing between calls[^neefay]. Touches `CR-21`, `CR-20`, `CR-27`.
- A status tool that says whether the game answers and whether a mission is running, because "only the main menu has nothing that answers"[^headswe] and the executor starts only after a mission loads[^zihuange]. Errors name the cause ("No answer from the game ... not polling") instead of a bare timeout[^headswe]. Touches `CR-27`, `CR-30`, `CR-06`.
- A guide tool for clients that do not load instruction files[^neefay], like Maestro's cheat sheet[^mae-mcp]. Touches `CR-53`.
- Loopback only, token per run, BattlEye off, and an audit log of every executed request[^neefay][^headswe]. Touches `CR-60`, `CR-61`.

## Recommended transport and security shape

A recommendation for the spec, not a decision:

1. **Agent to framework: stdio.** The MCP server is launched by the agent's client from the project's `.mcp.json`. stdio confines access to the launching client and needs no OAuth[^mcp-security][^mcp-auth]. A Streamable HTTP mode is not needed for the MVP; if it comes later it binds 127.0.0.1, validates `Origin`, and requires a bearer token[^mcp-http].
2. **Framework to session: a long-lived runner, outside the agent's process tree.** The MCP server and the command line are thin clients of a runner that owns the game processes, so ending an agent session does not end the game session, and the runner's own job object still stops everything on a crash (`CR-01`, `CR-05`).
3. **Runner to game: loopback with a token per run.** The channel into the game (T3 picks it) listens only on 127.0.0.1, requires the run's token before any command, rejects anything that is not its protocol, and activates only for a game process the runner started or attached to (Neefay's PID session file)[^neefay]. The token travels through the environment or a user-only file, never on a command line other users can read. Every executed request goes to an audit log with the run.
4. **Tool shape.** One prefix; a small core: `status`, `start_session`, `stop_session`, `exec` (code, machine, args), `inspect` (a compact state summary of a machine or around an object), `events` (cursor), `wait_for` (condition or event, ceiling), `expect` (live assertion), `list_checks` (with tags), `run_checks` (returns a run handle), `run_status` and `run_result` (by handle), `reload`, `logs` (machine, filter), `errors`, `guide`. Read-only tools are marked so. Each has an `outputSchema`; a failed check is a successful call with a verdict; `isError` is for what the agent can fix, and names the fix. Every call has a ceiling below the client's timeout, sends progress while it waits, and stops game-side work when cancelled. Results stay under the client's output limit and point to files for the rest. Every tool has a command-line twin with the same JSON.

**Needs a spike**: pass condition, an agent session in Claude Code on the test machine starts a session through the MCP server, the agent session ends, and the game session is still up and answers a `status` from a new agent session.

[^mcp-spec]: MCP specification, latest revision page.
[^mcp-changelog]: MCP specification 2026-07-28, Key changes.
[^mcp-transports]: MCP specification 2026-07-28, Transports overview.
[^mcp-stdio]: MCP specification 2026-07-28, stdio transport.
[^mcp-http]: MCP specification 2026-07-28, Streamable HTTP transport, security warning.
[^mcp-progress]: MCP specification 2026-07-28, Progress.
[^mcp-cancel]: MCP specification 2026-07-28, Cancellation and timeouts.
[^mcp-tools]: MCP specification 2026-07-28, Tools: handles, structured content and error handling.
[^mcp-tasks]: MCP Tasks extension overview.
[^mcp-auth]: MCP specification 2026-07-28, Authorization.
[^mcp-security]: MCP specification 2026-07-28, Security best practices: local MCP server compromise.
[^cc-mcp]: Claude Code docs, Connect Claude Code to tools via MCP.
[^pw-mcp]: Playwright MCP README at the named commit.
[^mae-mcp]: Maestro docs, Maestro MCP server.
[^headswe]: `headswe/arma3-mcp` README at the named commit.
[^neefay]: `Neefay/arma3-llm-harness` README at the named commit.
[^zihuange]: `Zihuange/arma3-mcp` README, tool adapters and safety doc at the named commit.
[^a3bridge]: `lepitinox/a3-bridge` README at the named commit.
[^a3ref]: `a3ref/mcp` README at the named commit.
[^wikimcp]: `ScottTMConnors/arma3-wiki-mcp` README at the named commit.
