---
type: Documentation
title: Recording the mod's events
description: How a check records a CBA event or an engine mission event on the machines it names with skd_fnc_record, reads every event with its arguments and clocks with skd_fnc_events, waits for one with skd_fnc_waitEvent, how recording ends with the check, where the result file lists the events, and how each function fails.
status: draft
---

# Recording the mod's events

Many mods announce what they do through events: a CBA event raised when a rope attaches, an engine event when a unit dies. A check records those events on the machines it names, reads what each machine received with the clocks of each, and waits for one, without writing event handlers of its own. The reference is [The check API](check-api.md#recording-events).

An event is a **CBA event**, raised with `CBA_fnc_localEvent`, `CBA_fnc_globalEvent`, `CBA_fnc_serverEvent` or another of CBA's event functions, or an **engine mission event**, such as `EntityKilled`, that `addMissionEventHandler` takes. Its name is matched without regard to case.

## An example

```sqf
// @check US-072 p2 hears the signal p1 gives
// @setups dedicated
["all", "mymod_signalled"] call skd_fnc_record;              // a CBA event, on every machine
["server", "EntityKilled", "mission"] call skd_fnc_record;   // a mission event, on the server

["p1", "mymod", "signal"] call skd_fnc_pressKey;             // the mod raises mymod_signalled
private _args = ["p2", "mymod_signalled", 10] call skd_fnc_waitEvent;
[_args select 0, "==", ["p1"] call skd_fnc_player, "the signal names p1's unit"] call skd_fnc_expect;

private _heard = ["server", "mymod_signalled"] call skd_fnc_events;
[count _heard, "==", 1, "the server heard it once"] call skd_fnc_expect;
["first signal on the server", (_heard select 0) get "at"] call skd_fnc_evidence;
```

## Recording: `skd_fnc_record`

`[machines, event, kind, ceiling] call skd_fnc_record` starts recording the event on each machine named, a machine name, an array of names or a group. `kind` is `"cba"`, the default, or `"mission"`. It returns `true` once every machine records it; recording starts then.

Recording an event already recorded on a machine keeps what it recorded. It fails `fail` / `check-error`:

- for a `"cba"` event on a machine where CBA is not loaded;
- for a `"mission"` event the engine does not have;
- when the event is recorded there already as the other kind.

## Reading: `skd_fnc_events`

`[machine, event] call skd_fnc_events` returns the events of that name recorded on the machine, oldest first. Each is a HashMap:

| Key | Holds |
|---|---|
| `"args"` | The event's arguments as the machine received them. Objects travel by net id, so an object the lead does not know arrives as `objNull`. |
| `"at"` | The machine's clocks when it came: a HashMap with `"utc"` (a String, the wall clock), `"tick"` (`diag_tickTime`), `"time"` (mission time) and `"frame"` (frame number). |

```sqf
private _all = ["p1", "mymod_signalled"] call skd_fnc_events;
{ diag_log [(_x get "at") get "time", _x get "args"] } forEach _all;
```

## Waiting: `skd_fnc_waitEvent`

`[machine, event, ceiling, after] call skd_fnc_waitEvent` returns the arguments of event number `after + 1` on the machine once it is recorded. It returns at once when that event was recorded before the call, so a check that triggers the event and then waits for it does not miss it: trigger first, wait after. To wait for the next event after those it has seen, pass their count as `after`:

```sqf
["p1", "mymod", "signal"] call skd_fnc_pressKey;
["p1", "mymod_signalled", 5] call skd_fnc_waitEvent;      // the first
["p1", "mymod", "signal"] call skd_fnc_pressKey;
["p1", "mymod_signalled", 5, 1] call skd_fnc_waitEvent;   // the second
```

When the event does not come, the check ends `fail` / `wait-ceiling` with what did arrive, on the machine waited on and on every other machine the check records on:

```
skd_fnc_waitEvent: no event mymod_signalled came on p2 within its ceiling of 3 s. Recorded events: p2: none; p1: 1 mymod_signalled.
```

`evidence.observed` holds the same counts per machine.

A wait or `skd_fnc_events` for an event the check does not record on that machine ends it `fail` / `check-error`:

```
skd_fnc_waitEvent: the check does not record the event mymod_signalled on p2 (it records it on p1). Record it first with skd_fnc_record.
```

## When recording ends

Recording ends with the check: every handler the check's recording added is removed on every machine when the check ends, whatever its outcome, so the next check records nothing until it records again and no handler is left behind.

## In the result file

The check's `events` lists every event recorded during the check, on every machine, in the order of their wall-clock time, with its `machine`, `name`, `kind` (`cba` or `mission`), `args` and its clocks `at`. It is empty when the check recorded none, and it is kept for every outcome ([Results and the result file](results.md)).
