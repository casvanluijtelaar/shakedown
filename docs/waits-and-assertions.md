---
type: Documentation
title: Waits and assertions
description: How a check waits for a condition and asserts a value on one machine, on several at once, or that machines agree, with no sleep - every operator, ceilings and how they are cut, how values from other machines reach the lead, and the message each failure gives.
status: draft
---

# Waits and assertions

Arma 3 is a live simulation: a rope takes a frame to appear, a value takes a moment to reach a client. Shakedown has no sleep. Every wait has a condition and a ceiling, and passes the moment the condition holds. The reference is [The check API](check-api.md#waits).

## Waits

```sqf
["p1", { player getVariable ["agh_climbState", "OFF"] == "OFF" }, "p1 is off the rope", 5] call skd_fnc_waitUntil;
private _landed = ["p1", { isTouchingGround player }, "p1 landed", 3] call skd_fnc_tryWaitUntil;
if (!_landed) then { ["landing", "p1 was still in the air"] call skd_fnc_evidence };
```

| Function | At the ceiling |
|---|---|
| `skd_fnc_waitUntil [machine, condition, description, ceiling, args]` | Ends the check `fail` / `wait-ceiling` with the description. |
| `skd_fnc_tryWaitUntil [machine, condition, description, ceiling, args]` | Returns `false`; the check goes on. |

- The condition is evaluated on the named machine every frame, from the first frame, until it returns `true` or the ceiling passes.
- `machine` is one machine, not a group. To wait on several, use `skd_fnc_expectEach` with `"=="` and `true`.
- Write the description as the state you wait for, so the failure reads well:

  ```
  skd_fnc_waitUntil: "p1 is off the rope" was not met on p1 within its ceiling of 5 s.
  ```

- A condition that raises a script error ends the check at that evaluation, with no retry, `fail` / `script-error`:

  ```
  skd_fnc_waitUntil: the condition for "p1 is off the rope" on p1 raised a script error: Undefined variable in expression: _state. Fix the condition or the code it calls.
  ```

- A condition that returns anything but `true` or `false` ends it `fail` / `check-error`, naming what it returned:

  ```
  skd_fnc_waitUntil: the condition for "p1 is off the rope" on p1 returned a String ("OFF"), not a Boolean. Make it return true or false.
  ```

- `skd_fnc_tryWaitUntil` ends the check the same two ways; it returns `false` only at its ceiling.

## Assertions

```sqf
[_n, "==", 0, "no box is left"] call skd_fnc_expect;
["p1", { damage player }, "<", 0.1, "p1 is unhurt"] call skd_fnc_expectOn;
["all", { count ropes (_this select 0) }, ">=", 1, "every machine sees the rope", 10, [_anchor]] call skd_fnc_expectEach;
["all", { getPosASL (_this select 0) }, "every machine agrees where p1 is", 0.1, 10, [["p1"] call skd_fnc_player]] call skd_fnc_expectAgree;
```

| Function | Does |
|---|---|
| `skd_fnc_expect [actual, op, expected, description]` | Asserts at once, on values the body already has. |
| `skd_fnc_expectOn [machine, value, op, expected, description, ceiling, args]` | Reads `value` on a machine every frame until the comparison holds, and returns the last value. |
| `skd_fnc_expectEach [machines, value, op, expected, description, ceiling, args]` | The same on several machines; holds when the comparison holds on every machine at once. Returns a HashMap of their last values. |
| `skd_fnc_expectAgree [machines, value, description, tolerance, ceiling, args]` | Holds when every machine reads the value the first machine of the list reads, within `tolerance` for numbers, and for positions (arrays of 2 or 3 numbers) by distance. Returns a HashMap of their last values. |

A polled assertion is a wait and an assertion in one: prefer `skd_fnc_expectOn` to a `skd_fnc_waitUntil` followed by `skd_fnc_expect`. It passes as soon as the comparison holds.

### The operators

| `op` | Holds when |
|---|---|
| `"=="` | `actual isEqualTo expected`: case-sensitive, works for every type. |
| `"!="` | `!(actual isEqualTo expected)`. |
| `"<"`, `"<="`, `">"`, `">="` | Both are numbers and the comparison holds. |
| `"~="` | `expected` is `[target, tolerance]`; `actual` is a number within `tolerance` of `target`, or a position within `tolerance` metres of it. |
| `"in"` | `actual` is an element of the array `expected`, by `isEqualTo`. |
| `"contains"` | `expected` is an element of the array `actual`, or a substring of the string `actual`, case-sensitive. |

```sqf
["p1", { getPosASL player }, "~=", [AGLToASL [100, 104, 0], 0.5], "p1 stands at the wall"] call skd_fnc_expectOn;
["p1", { animationState player }, "in", ["amovpercmstpsnonwnondnon", "amovpercmstpsraswrfldnon"], "p1 stands"] call skd_fnc_expectOn;
["p1", { items player }, "contains", "FirstAidKit", "p1 carries a first aid kit"] call skd_fnc_expectOn;
[name (["p1"] call skd_fnc_player), "contains", "p1", "p1's name"] call skd_fnc_expect;
```

A position compared with `~=` is measured in three dimensions, so the target is ASL like the value read: write a position on the ground with `AGLToASL`. On the default world `VR` the ground is about 5 m above sea level, so a player standing at `[100, 104]` reads `[100, 104, 5.00144]` and a target of `[100, 104, 0]` is 5 m away.

Any other `op`, or operands of the wrong type for it, ends the check `fail` / `check-error`:

```
skd_fnc_expectOn: operator < needs two numbers; "p1's state" read "OFF" and expected 1.
```

## What a failed assertion says

At once, for `skd_fnc_expect`:

```
no box is left: expected == 0, got 2.
```

At the ceiling, for `skd_fnc_expectOn`, with the last value it read:

```
p1 is unhurt: expected < 0.1 on p1 within 10 s, last read 0.35.
```

For `skd_fnc_expectEach`, with every machine's last value and the machines that did not meet it:

```
every machine sees the rope: expected >= 1 on every machine within 10 s; last read server 1, p1 1, p2 0; not met on p2.
```

For `skd_fnc_expectAgree`, with what each machine that differs read where the first machine read its value:

```
the crate's flag: the machines did not agree within 2 s (tolerance 0): p2 read 6 where server read 5. Every machine read: server 5, p1 5, p2 6.
```

Every one ends the check `fail` / `assertion`. The result file keeps each machine's last value under `evidence.observed`, and `reason.machine` names the first machine that did not meet it, or the first that differs. A machine whose code returned nothing reads `nil`.

## Values from other machines

The body runs on the lead machine. A wait or polled assertion on another machine evaluates its code there every frame and sends each new value back through the runner, at most one every 0.05 s; the lead compares each machine's newest value. A round trip to another machine takes about 12 ms to a game client and about 43 ms to the server or a headless client, which run at a lower frame rate.

Objects in `args` are looked up again before every evaluation, so a wait finds an object the moment it reaches the machine, and reads `objNull` until then. That makes a wait the way to make sure another machine knows an object before you act on it there ([Code, waits and agreement across machines](dedicated-sessions.md#code-waits-and-agreement-across-machines)).

## Ceilings

- Every wait and polled assertion takes a `ceiling` in seconds of real time, default 10. A wait given no ceiling ends at 10 s.
- A ceiling is a number above 0; `nil` takes the default, and anything else ends the check `fail` / `check-error`.
- A ceiling that reaches past the check's `@timeout` is cut to the time left, and the check then ends `fail` / `check-timeout`.
- The runner waits the ceiling plus 10 s on its own clock. A machine that has not answered by then is lost, and the check is `invalid` / `machine-lost`, not `fail`: a wait that timed out because its machine stopped answering is not the mod's fault ([Outcomes and a broken session](outcomes-and-broken-sessions.md)).

Choose a ceiling that is generous for a healthy game and short enough that a failure is not slow: a few seconds for something that should happen at once, longer for something that runs through the network or a long animation.

## Trying a wait or an assertion live

Between runs, `shakedown wait` and `shakedown expect` try the same thing in a running session without writing a check ([Asking a running game](asking-a-running-game.md)):

```
shakedown wait --machine p1 --condition "isTouchingGround player" --ceiling 5
shakedown expect --machine p1 --value "damage player" --op "<" --expected 0.1
```
