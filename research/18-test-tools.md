---
type: Research
title: Test automation tools for the web, mobile and games
description: How Playwright, Maestro and the game test tools shape a test, wait on a running app, retry, filter and report, and what of each fits a check written in SQF.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T21:55:00+01:00 }
sources:
  - { id: pw-release, resource: "https://github.com/microsoft/playwright/releases (v1.64.1, 2026-10-14)", read: 2026-11-04 }
  - { id: pw-actionability, resource: "https://playwright.dev/docs/actionability", read: 2026-11-04 }
  - { id: pw-assertions, resource: "https://playwright.dev/docs/test-assertions", read: 2026-11-04 }
  - { id: pw-timeouts, resource: "https://playwright.dev/docs/test-timeouts", read: 2026-11-04 }
  - { id: pw-locators, resource: "https://playwright.dev/docs/locators", read: 2026-11-04 }
  - { id: pw-fixtures, resource: "https://playwright.dev/docs/test-fixtures", read: 2026-11-04 }
  - { id: pw-retries, resource: "https://playwright.dev/docs/test-retries", read: 2026-11-04 }
  - { id: pw-annotations, resource: "https://playwright.dev/docs/test-annotations", read: 2026-11-04 }
  - { id: pw-cli, resource: "https://playwright.dev/docs/test-cli", read: 2026-11-04 }
  - { id: pw-step, resource: "https://playwright.dev/docs/api/class-test (test.step)", read: 2026-11-04 }
  - { id: pw-trace, resource: "https://playwright.dev/docs/trace-viewer", read: 2026-11-04 }
  - { id: pw-reporters, resource: "https://playwright.dev/docs/test-reporters", read: 2026-11-04 }
  - { id: pw-agents, resource: "https://playwright.dev/docs/test-agents", read: 2026-11-04 }
  - { id: pw-mcp, resource: "https://github.com/microsoft/playwright-mcp README at commit 50b004fb3d3ee17f9710f067ed1b6d59fd16e4a1 (release v0.0.84)", read: 2026-11-04 }
  - { id: mae-release, resource: "https://github.com/mobile-dev-inc/maestro/releases (cli-2.12.1, 2026-11-02)", read: 2026-11-04 }
  - { id: mae-how, resource: "https://docs.maestro.dev/get-started/how-maestro-works.md", read: 2026-11-04 }
  - { id: mae-wait, resource: "https://docs.maestro.dev/reference/commands-available/extendedwaituntil.md", read: 2026-11-04 }
  - { id: mae-retry, resource: "https://docs.maestro.dev/reference/commands-available/retry.md", read: 2026-11-04 }
  - { id: mae-tags, resource: "https://docs.maestro.dev/maestro-flows/workspace-management/test-discovery-and-tags.md", read: 2026-11-04 }
  - { id: mae-hooks, resource: "https://docs.maestro.dev/maestro-flows/flow-control-and-logic/hooks.md", read: 2026-11-04 }
  - { id: mae-reports, resource: "https://docs.maestro.dev/maestro-flows/workspace-management/test-reports-and-artifacts.md", read: 2026-11-04 }
  - { id: mae-mcp, resource: "https://docs.maestro.dev/get-started/maestro-mcp.md", read: 2026-11-04 }
  - { id: utf-unitytest, resource: "https://docs.unity3d.com/Packages/com.unity.test-framework@1.4/manual/reference-attribute-unitytest.html (Test Framework 1.4.6)", read: 2026-11-04 }
  - { id: utf-domainreload, resource: "https://docs.unity3d.com/Packages/com.unity.test-framework@1.4/manual/reference-wait-for-domain-reload.html", read: 2026-11-04 }
  - { id: utf-logassert, resource: "https://docs.unity3d.com/Packages/com.unity.test-framework@1.4/manual/reference-custom-assertion.html", read: 2026-11-04 }
  - { id: ue-framework, resource: "https://dev.epicgames.com/documentation/en-us/unreal-engine/automation-test-framework-in-unreal-engine (UE 5.8)", read: 2026-11-04 }
  - { id: ue-cpp, resource: "https://dev.epicgames.com/documentation/en-us/unreal-engine/write-cplusplus-tests-in-unreal-engine (UE 5.8)", read: 2026-11-04 }
  - { id: ue-spec, resource: "https://dev.epicgames.com/documentation/en-us/unreal-engine/automation-spec-in-unreal-engine", read: 2026-11-04 }
  - { id: ue-gauntlet, resource: "https://dev.epicgames.com/documentation/en-us/unreal-engine/gauntlet-automation-framework-in-unreal-engine (UE 5.8)", read: 2026-11-04 }
  - { id: ue-gauntlet-ctrl, resource: "https://dev.epicgames.com/documentation/en-us/unreal-engine/gauntlet-controller-in-unreal-engine (UE 5.8)", read: 2026-11-04 }
  - { id: alt-commands, resource: "https://alttester.com/docs/sdk/latest/pages/commands.html (SDK 2.3.2)", read: 2026-11-04 }
  - { id: gd-start, resource: "https://kb.gamedriver.io/getting-started-with-gamedriver (served as kb.gamedriver.ai)", read: 2026-11-04 }
  - { id: gut-await, resource: "https://gut.readthedocs.io/en/latest/Awaiting.html", read: 2026-11-04 }
  - { id: cba-tests, resource: "https://github.com/CBATeam/CBA_A3 at commit 8b15a04bd52ff7acc568a93cf6e90343e43a8a56, addons/arrays/test.sqf, addons/arrays/test_join.sqf, addons/main/script_macros_common.hpp", read: 2026-11-04 }
---

# Test automation tools for the web, mobile and games

This topic answers what the check API, the runner and the report should feel like (`CR-31`, `CR-32`, `CR-40`, `CR-42`, `CR-50`, `CR-51`). It reads Playwright and Maestro for the shape of a test and its tooling, and the game test tools for how they drive a running game and wait on it. It ends with candidate words for an SQF check API. They are candidates, not decisions: the spec picks them.

Versions read: Playwright 1.64.1[^pw-release], Playwright MCP 0.0.84[^pw-mcp], Maestro CLI 2.12.1[^mae-release], Unity Test Framework 1.4.6[^utf-unitytest], Unreal Engine 5.8 docs[^ue-framework], AltTester SDK 2.3.2[^alt-commands].

## Playwright

### How a test reads

A Playwright test is a named function that takes the fixtures it needs, acts on the page and asserts with `expect`. Named steps group the actions: `test.step(title, body, options)` shows in the report and the trace viewer, can be nested, and returns the value of its body[^pw-step]. A step can carry a `timeout` (default none) and throws a `TimeoutError` when it runs past it; `box: true` points an error inside the step at the step's call site[^pw-step].

**Forces:** a check is a named block made of named steps, and each step's name, duration and error are recorded. This is the unit `CR-42` (time per step) and `CR-23` (error traced to its step) need. Touches `CR-50`, `CR-42`, `CR-23`.

### Auto-waiting and actionability

Before an action Playwright waits until the target passes the checks that action needs: visible, stable (same bounding box for two animation frames), enabled, editable, and receives events. A click needs visible, stable, receives events and enabled. If the checks do not pass within the timeout, the action fails with a `TimeoutError`[^pw-actionability].

Assertions retry too. `expect(locator).toHaveText()` and its kin re-test until the condition holds or the timeout, 5 seconds by default, runs out[^pw-assertions]. `expect.poll(fn)` turns any function into a polled assertion, with default intervals of 100, 250, 500 and 1000 ms and a 5 second timeout; `expect(...).toPass()` retries a whole block, with a default timeout of 0[^pw-assertions]. A soft assertion records a failure and lets the test go on[^pw-assertions]. Every test has a ceiling, 30 seconds by default, that covers the test body, its fixtures and its `beforeEach` hooks[^pw-timeouts].

**Forces:** waits are the default, not something the author adds. Every wait and every assertion that reads game state polls until it holds or hits a ceiling, and the check as a whole has a ceiling. A fixed sleep has no place in the API (the working rule and `CR-20`). On timeout the failure carries the last observed value, as scenario "things go wrong" 1 of the PRD asks. Touches `CR-20`, `CR-31`.

What "actionable" means in Arma has no ready answer: a player can only use an action when the action's condition is true, the player is close enough and alive, and the action menu is not blocked. The framework needs its own actionability checks for a mod's actions and key handlers. That belongs to T6. Touches `CR-12`.

### Locators

A locator is a way to find an element "at any moment": each time it is used, Playwright finds the up-to-date element again, so a re-rendered page does not break it[^pw-locators]. Playwright recommends locators by user-facing role and name (`getByRole`) over CSS and XPath, which break when the DOM changes. Locators are strict: an action on a locator that matches several elements throws[^pw-locators].

**Forces:** the game has no DOM, but the idea carries over. A check should name a machine and an object by a stable handle (a player by name, an object by a variable name or `netId`) that is resolved again at each use, because objects can be deleted and recreated and locality moves. A handle that matches nothing, or more than one thing when one is expected, is an error in the check, not a failed verdict. Touches `CR-10`, `CR-22`, `CR-31`.

### Fixtures and setup

A fixture sets up what a test needs, hands it over with `use()` and tears it down after, so setup and teardown sit together. Test-scoped fixtures are made for each test; worker-scoped fixtures are made once per worker process and shared, which suits expensive resources such as servers. Only the fixtures a test asks for are set up. Fixtures can depend on each other and can be automatic[^pw-fixtures].

**Forces:** two scopes map directly onto the PRD. The session (game, server, clients) is the worker-scoped fixture: made once, shared by many checks (`CR-01`). The scene a check builds is test-scoped and torn down after the check (`CR-02`). Teardown runs whether the check passed or failed. Touches `CR-01`, `CR-02`, `CR-11`, `CR-41`.

### Retries and flaky reporting

Retries are off by default. With retries on, a test that fails and then passes is reported as **flaky**, distinct from passed and failed. After a failure Playwright throws away the whole worker process with its browser and starts a new one before the retry. A test can read `testInfo.retry` to know it is a retry[^pw-retries]. `--repeat-each` runs every test N times, and `--fail-on-flaky-tests` fails the run if any test is flaky[^pw-cli].

**Forces:** `CR-32` is `--repeat-each` with an outcome count, and the PRD's "passes once in three runs" scenario is Playwright's flaky outcome. Two lessons carry over: a flaky result is its own label in the report, never folded into pass; and a retry after a failure starts from a clean state. In Arma a full restart costs a game start, so "clean" has to mean a reset in place (T5) unless the failure made the session unhealthy, in which case the check is invalid (`CR-30`, `CR-31`). Touches `CR-31`, `CR-32`, `CR-30`.

Playwright has no third outcome like the PRD's **invalid**. Its closest is a timeout or a crashed worker, which it reports as failed. The framework adds invalid on top of the Playwright shape. Touches `CR-31`.

### Annotations, tags and filters

`test.skip()`, `test.fail()` (expected to fail; Playwright complains if it passes), `test.fixme()` and `test.slow()` (triples the timeout) annotate a test, and can be conditional[^pw-annotations]. Tags start with `@` and sit in the title or a `tag` option; `--grep @fast` and `--grep-invert @fast` filter by them, and custom annotations carry a `type` and a `description`, such as a link to an issue[^pw-annotations]. The command line also runs one file at a line (`file:42`), only the last failures (`--last-failed`), a named project (`--project`), stops after N failures (`--max-failures`), and lists tests without running them (`--list`)[^pw-cli].

**Forces:** `CR-40` (run the checks that name a story or a finding) is tags plus grep. A story ID such as `US-012` and a finding ID are tags on a check. `test.fail()` is the shape of `CR-33` (a check that must fail against a broken build). A Playwright "project" is a named configuration a test runs under, which is the shape of `CR-41` (setups per check: singleplayer, dedicated with two clients). `--list` matters for an agent: it asks what exists before it runs. Touches `CR-40`, `CR-41`, `CR-33`, `CR-53`.

### Trace and report output

A trace records each action with its locator and duration, before and after snapshots, the console, the network, the source line and the errors, on a timeline[^pw-trace]. Tracing every test is heavy, so the advice is `trace: 'on-first-retry'` or `retain-on-failure`[^pw-trace]. Built-in reporters print a line per test (`list`), one line (`line`), one character per test (`dot`), a self-contained HTML folder, JSON, JUnit XML, a mergeable blob, and GitHub annotations; several can run at once, and a custom reporter implements `onBegin`, `onTestBegin`, `onTestEnd` and `onEnd`[^pw-reporters].

**Forces:** `CR-51` is the JSON reporter's job: one machine-readable file per run with setup, checks, outcomes, step times and evidence. A human summary on the console comes from the same events. The trace idea becomes the evidence of a failed check: the step timeline, the last observed values, the script errors and the log lines of each machine around the failure, kept on failure and not on every pass, to keep runs light. JUnit XML can wait; nothing in the PRD asks for CI. Touches `CR-51`, `CR-24`, `CR-23`, `CR-42`.

### How agents use Playwright

Playwright ships three agents: a planner that explores the app and writes a Markdown test plan, a generator that turns the plan into test files and checks selectors live, and a healer that runs the suite and repairs failing tests in a loop. `npx playwright init-agents --loop=claude` (also `vscode`, `codex`, `opencode`) writes the agent definitions, which are "collections of instructions and MCP tools", and the docs say to regenerate them when Playwright updates[^pw-agents]. The Playwright MCP server lets an agent drive a browser through accessibility snapshots instead of screenshots; it also notes that coding agents increasingly prefer a CLI with skills because it costs fewer tokens than large tool schemas and accessibility trees[^pw-mcp]. T10 reads its tool shapes.

**Forces:** an agent writes and repairs checks from a live session: it looks at state, tries a step, then writes the check. The framework needs both a running session an agent can poke at (`CR-27`) and a command line whose output is short and structured, since an agent may use either. Touches `CR-53`, `CR-27`.

## Maestro

Maestro writes a flow as YAML that reads "as English instructions", and the CLI interprets the YAML and sends instructions to the device, so a flow needs no build step[^mae-how]. It drives the app "at arm's length" through the accessibility tree and low-level taps, swipes and text input, using a driver app on the device[^mae-how]. It says it "embraces instability": it waits for the screen to settle or content to load, so a flow needs no `sleep()`[^mae-how].

Waits have a default ceiling of 7 seconds; `extendedWaitUntil` with `visible` or `notVisible` and a `timeout` in milliseconds waits longer and fails at its ceiling[^mae-wait]. `retry` reruns a block of commands up to `maxRetries` (0 to 3, default 1), and the docs warn that wrapping large parts of a flow hides real flakiness[^mae-retry]. Tags sit in the flow's header; `--include-tags` and `--exclude-tags` filter, and `config.yaml` can set a workspace policy such as always excluding `flaky`[^mae-tags]. `onFlowStart` and `onFlowComplete` hooks run setup and teardown; if `onFlowStart` fails the flow is failed and its body skipped, and `onFlowComplete` still runs[^mae-hooks].

Reports come as JUnit XML or HTML. Each run gets a folder with a screenshot of the failing step, the view hierarchy at that step, the logs, and a `commands.json` with one entry per executed step: command, status, duration, error and artifacts, with a retried step listed once per attempt. A `manifest.json` indexes every artifact[^mae-reports].

The Maestro MCP server runs over stdio from `maestro mcp` and has few tools: list devices, inspect the screen as compact JSON, take a screenshot, run flows (inline YAML, files or a folder with tag filters), a cheat sheet of commands, and cloud runs[^mae-mcp].

**Forces:**
- A check in a language that is interpreted, not built, gives a fast edit-and-check loop. SQF compiles at run time, so a check file can be sent into a running session and run at once. Touches `CR-03`, `CR-50`, G2.
- A default ceiling on every wait, raised per wait. Touches `CR-20`.
- Setup that fails is not the check failing. Maestro marks it failed; the PRD's third outcome lets the framework mark a check whose setup failed for an environment reason as invalid. Teardown always runs. Touches `CR-31`, `CR-02`.
- A per-step log (`commands.json`) with status, duration, error and artifacts is a ready model for the steps inside the result file, and the manifest for its evidence. Touches `CR-51`, `CR-42`.
- An agent tool surface can be small: inspect, run, a cheat sheet. Touches `CR-53`.

## Game test tools

### Unity Test Framework

`[UnityTest]` extends NUnit so a test can pause partway. In Play Mode the test runs as a coroutine on a MonoBehaviour: `yield return null` skips a frame, `WaitForSeconds` pauses for a time. Edit Mode tests run in the editor's update loop[^utf-unitytest]. `WaitForDomainReload` holds the test until a code reload finishes and throws if the reload fails to compile[^utf-domainreload]. A test fails when Unity logs anything other than a regular log or a warning; `LogAssert.Expect` declares an expected error, `LogAssert.ignoreFailingMessages` turns the check off, and `LogAssert.NoUnexpectedReceived` asserts that nothing unexpected came[^utf-logassert]. The docs read show no timeout for a `[UnityTest]`; Unity's NUnit timeout is **unverified** here.

**Forces:**
- A test that runs inside the game's own loop and yields frames is the shape SQF already has: scheduled code with `waitUntil` and `sleep`. The framework's waits are scheduled SQF that yields, and the ceiling has to be the framework's, since the engine has none. Touches `CR-20`, `CR-50`.
- A script error fails the check unless the check says it expects it. That is the default `CR-23` wants, with an `expectError` escape for checks about errors. Touches `CR-23`, `CR-31`.
- A wait on "the reload finished", failing on a compile error, is the shape of `CR-03`. Touches `CR-03`.

### Unreal Automation System and Gauntlet

Unreal's automation tests come as unit, feature, smoke, content stress and screenshot comparison tests; smoke tests "are intended to complete within 1 second"[^ue-framework]. Work that spans frames goes in latent commands: `DEFINE_LATENT_AUTOMATION_COMMAND` defines one, `ADD_LATENT_AUTOMATION_COMMAND` queues it, and it is complete when its `Update` returns true; returning false retries on the next frame[^ue-cpp]. Test flags such as `EditorContext` and `EngineFilter` set a test's requirements; the full list was not reachable (the reference link returned a 404)[^ue-cpp]. The Automation Spec format writes tests as `Describe`, `It`, `BeforeEach` and `AfterEach`, with `LatentIt` taking a `Done` delegate that the test waits on[^ue-spec]. Timeouts for latent commands are **unverified**: the pages read do not state them.

Gauntlet runs sessions of a project that "perform tests and validate results", across platforms, with several processes: "a multiplayer game may require four clients and a server"[^ue-gauntlet]. Inside the game an optional `TestController` puppeteers and monitors the instance: `OnTick` is called periodically to check and drive state, `OnStateChange` fires on game-driven state changes, and `EndTest(ExitCode)` hands the result to the outer runner, which promotes it to the test's result[^ue-gauntlet-ctrl]. How the outer runner watches processes and times them out is **unverified**: the pages read do not say.

**Forces:**
- Gauntlet is the nearest model for the framework: an outer runner starts a session of several game processes (server and clients), and a controller inside the game drives the test and reports back. The split matters: the outer runner owns processes, ceilings and the verdict when the game stops answering (invalid); the inner controller owns steps, waits and assertions. Touches `CR-01`, `CR-05`, `CR-30`, `CR-31`.
- A latent command polled each frame until it returns true is a wait with a condition. Touches `CR-20`.
- Smoke tests with a time budget suggest a tag such as `smoke` for a fast subset. Touches `CR-40`.

### AltTester

AltTester instruments a Unity build so a driver outside it can find and act on objects. The driver connects over WebSocket (`host` 127.0.0.1, `port` 13000 by default, `wss` with `secureMode`) to the AltTester Server, with a 60 second connect timeout[^alt-commands]. `WaitForObject(by, value, ..., timeout, interval)` polls until an object matching the criteria appears or the timeout runs out, 20 seconds by default; `WaitForObjectNotBePresent` waits for it to go[^alt-commands].

**Forces:** the test lives outside the game and sends finds and waits in. Each wait is a round trip polled from outside. For Arma, a check in SQF runs inside the game and polls there, which avoids a round trip per poll. External polling stays useful for the runner's health checks. Touches `CR-20`, `CR-30`.

### GameDriver

GameDriver's agent sits in the game and the test connects to it: `api.Connect("localhost", 19734, ...)`, then `api.WaitForObject("//*[@name='StartButton']")` with an XPath-like HierarchyPath, then `api.ClickObject(...)` with a timeout; input hooking is turned on with `api.EnableHooks(HookingObject.ALL)`[^gd-start]. The default timeout of `WaitForObject` is **unverified**: the page read does not state it.

**Forces:** input sent through hooks inside the game rather than the desktop is the shape `CR-15` and the "never take over the desktop" rule ask for. Whether Arma allows an equivalent is T6's question. Touches `CR-15`.

### Godot GUT

GUT tests await in the game loop: `wait_seconds(time)`, `wait_physics_frames(n)`, `wait_idle_frames(n)`, `wait_for_signal(sig, max_wait)` and `wait_until(callable, max_wait)`. `wait_until` checks the callable every frame, or at an interval, and needs it to return exactly `true`; both return true if the condition came in time and false on timeout, and the test goes on either way[^gut-await].

**Forces:** a wait that returns whether it succeeded, rather than failing the check, is useful for "did this happen within N seconds?" branches. The framework can offer both: a wait that fails the check at its ceiling, and a wait that returns a boolean. Waiting for an event (a signal) is as common as waiting for a condition, which matches CBA events in Arma. Touches `CR-20`, `CR-21`.

### How Arma mods test today

CBA ships SQF tests inside its addons: a `test.sqf` per addon runs each `test_*.sqf` with `execVM` and `waitUntil { scriptDone _test }`, and each test checks results with macros such as `TEST_OP(_result,==,_expected,_fn)`, `TEST_TRUE`, `TEST_FALSE` and `TEST_DEFINED`, which log "Test OK" or raise an error titled "Test FAIL"[^cba-tests]. These are unit tests of functions inside the running game, with no runner, no ceilings and no result file. T12 covers what developers say about testing.

**Forces:** Arma developers already read assertions as SQF macros and `waitUntil`. The check API should feel like that and add what is missing: ceilings, steps, a verdict per check and a result file. A macro-based API is a candidate, since CBA-based mods already include CBA's macro headers. Touches `CR-50`.

## What fits a check written in SQF

The comparison gives a short list of ideas the API and runner take, each with the tool it comes from:

| Idea | From | CR |
|---|---|---|
| A check is named with its story ID and made of named, timed steps | Playwright `test.step`, Maestro `commands.json` | `CR-50`, `CR-42` |
| Every wait and every state assertion polls to a ceiling; the check has a ceiling too | Playwright, Maestro, AltTester, GUT | `CR-20` |
| On timeout, report the last observed value | Playwright `expect.poll` | `CR-20`, `CR-31` |
| Session as a shared fixture, scene as a per-check fixture with guaranteed teardown | Playwright fixture scopes, Maestro hooks | `CR-01`, `CR-02` |
| Script errors fail the check unless expected | Unity `LogAssert` | `CR-23` |
| A third outcome, invalid, for setup and environment failures | none of them; the PRD adds it | `CR-31`, `CR-30` |
| Repeat N times; report flaky as its own label | Playwright `--repeat-each`, flaky | `CR-32` |
| Tags carry story and finding IDs; filter by tag | Playwright `--grep`, Maestro tags | `CR-40` |
| Named setups a check declares it runs in | Playwright projects | `CR-41` |
| Expected-to-fail mode | Playwright `test.fail()` | `CR-33` |
| An outer runner owns processes and the verdict when the game stops; a controller in the game owns steps | Unreal Gauntlet | `CR-05`, `CR-30` |
| One JSON result file per run; evidence kept on failure | Playwright JSON reporter and traces, Maestro artifacts | `CR-51` |

### A sketch of the check API vocabulary

These are candidate words, not decisions. Names are placeholders with a prefix (`AT_`) so they cannot clash with a mod's own names; the spec picks the real prefix and spellings. Each is an SQF function or a CBA-style macro.

| Candidate | Meaning | CR |
|---|---|---|
| `AT_fnc_check` with `["US-012 crate opens through the action", { ... }, ["tag1"], ["sp", "dedicated-2"]]` | Declares a check: its name (with the story ID), its body, its tags and the setups it runs in. | `CR-50`, `CR-40`, `CR-41` |
| `AT_fnc_step` with `["place a crate in front of player 1", { ... }]` | A named, timed step; returns its body's value. Errors inside it are traced to it. | `CR-42`, `CR-23` |
| `AT_fnc_on` with `[machine, { code }, args]` | Runs code on a machine and returns the result as data. `machine` is `"server"`, a player's name, `"clients"` or a headless client. | `CR-10` |
| `AT_fnc_onEach` with `[machines, { code }]` | Runs the same code on several machines and returns a map from machine to value. | `CR-22` |
| `AT_fnc_waitUntil` with `[{ condition }, ceiling, "what we wait for", machine]` | Polls until the condition is true or the ceiling passes; on timeout fails the check with the last value. A default ceiling applies when none is given. | `CR-20`, `CR-31` |
| `AT_fnc_waitFor` with `[eventName, ceiling, filter, machine]` | Waits for a mod or engine event, such as a CBA event, and returns its arguments. | `CR-20`, `CR-21` |
| `AT_fnc_tryWaitUntil` | Like `waitUntil` but returns true or false instead of failing. | `CR-20` |
| `AT_fnc_expect` with `[actual, "==", expected, "message"]`, and `AT_fnc_expectOn` with `[machine, { value }, "==", expected]` | An assertion; the machine form polls to the default ceiling, as Playwright's assertions do. | `CR-50`, `CR-22` |
| `AT_fnc_expectAll` with `[machines, { value }, "==", expected]` | Every named machine agrees on the value, polled to a ceiling. | `CR-22` |
| `AT_fnc_expectError` with `[pattern, { code }]` | Declares that a script error matching the pattern is expected inside the block, so it does not fail the check. | `CR-23` |
| `AT_fnc_fixture` with `["crate", { setup }, { teardown }]` | A per-check scene fixture whose teardown always runs. The session is the shared fixture the runner owns. | `CR-02`, `CR-11` |
| `AT_fnc_act` with `[player, actionTitleOrId]`, `AT_fnc_key` with `[player, keybind]` | Makes a player use a mod's action or fire a key handler through its real path, after the framework's own actionability checks. | `CR-12`, `CR-15` |
| `AT_fnc_skip`, `AT_fnc_expectFail` | Skip a check with a reason; mark a check that must fail, for `CR-33`. | `CR-33` |
| `AT_fnc_evidence` with `[label, value]` | Attaches a value, a screenshot path or a log excerpt to the check's evidence. | `CR-51`, `CR-25` |
| `AT_fnc_invalid` with `["reason"]` | Ends the check as invalid, not failed, for a cause outside the mod. The runner also sets invalid on its own when health fails. | `CR-31`, `CR-30` |

Open points for the spec, not settled here:
- Function calls (`[...] call AT_fnc_step`) read worse than macros (`STEP("...") { ... }`), but macros need a header include and a preprocessing step that a check sent into a running game may not get. Whether a check file sent at run time is preprocessed is a T4 and T3 question.
- Whether a check body runs scheduled (it must, to wait) and where: on the server, or on a framework controller that sends code to each machine. This decides the cost of `AT_fnc_on` and is T3's question. **Needs a spike**: pass condition, a scheduled check on the server runs a step on a named client and gets its result back within one second.
- The default ceiling. Playwright uses 5 seconds for assertions and 30 for a test[^pw-timeouts], Maestro 7 for waits[^mae-wait], AltTester 20[^alt-commands]. A game's default needs a measurement. **Needs a spike**: pass condition, the default ceiling covers the slowest common wait (an action's effect replicated to every client) on the test machine in 95 of 100 runs.

[^pw-release]: Playwright releases on GitHub, v1.64.1 dated 2026-10-14.
[^pw-actionability]: Playwright docs, Auto-waiting.
[^pw-assertions]: Playwright docs, Assertions.
[^pw-timeouts]: Playwright docs, Timeouts.
[^pw-locators]: Playwright docs, Locators.
[^pw-fixtures]: Playwright docs, Fixtures.
[^pw-retries]: Playwright docs, Retries.
[^pw-annotations]: Playwright docs, Annotations.
[^pw-cli]: Playwright docs, Command line.
[^pw-step]: Playwright API docs, `test.step`.
[^pw-trace]: Playwright docs, Trace viewer.
[^pw-reporters]: Playwright docs, Reporters.
[^pw-agents]: Playwright docs, Test agents.
[^pw-mcp]: Playwright MCP README at the named commit.
[^mae-release]: Maestro releases on GitHub, cli-2.12.1 dated 2026-11-02.
[^mae-how]: Maestro docs, How Maestro works.
[^mae-wait]: Maestro docs, `extendedWaitUntil`.
[^mae-retry]: Maestro docs, `retry`.
[^mae-tags]: Maestro docs, Test discovery and tags.
[^mae-hooks]: Maestro docs, Hooks.
[^mae-reports]: Maestro docs, Test reports and artifacts.
[^mae-mcp]: Maestro docs, Maestro MCP server.
[^utf-unitytest]: Unity Test Framework 1.4.6 manual, `UnityTest` attribute.
[^utf-domainreload]: Unity Test Framework manual, `WaitForDomainReload`.
[^utf-logassert]: Unity Test Framework manual, custom assertions and `LogAssert`.
[^ue-framework]: Unreal Engine 5.8 docs, Automation Test Framework.
[^ue-cpp]: Unreal Engine 5.8 docs, Write C++ tests.
[^ue-spec]: Unreal Engine docs, Automation Spec.
[^ue-gauntlet]: Unreal Engine 5.8 docs, Gauntlet Automation Framework.
[^ue-gauntlet-ctrl]: Unreal Engine 5.8 docs, Gauntlet Controller.
[^alt-commands]: AltTester SDK 2.3.2 docs, Commands.
[^gd-start]: GameDriver knowledge base, Getting started with GameDriver and Unity.
[^gut-await]: GUT docs, Awaiting.
[^cba-tests]: CBA_A3 at the named commit: the arrays test runner, one test, and the `TEST_*` macros.
