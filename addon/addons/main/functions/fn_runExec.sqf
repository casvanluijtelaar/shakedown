/*
    Runs an order of kind exec (P-03 step 2 and 3): compiles the source, spawns it with the
    arguments as _this, and answers with its value, or with state "ceiling" when the in-game
    ceiling passes first, or "compile-error" with the engine's message.

    Arguments: [order] - HashMap with "id", "source", "args", "ceiling".
*/
params ["_order"];
private _id = _order get "id";
private _ceiling = _order getOrDefault ["ceiling", 10];
private _args = _order getOrDefault ["args", []];
if (isNil "_args") then { _args = [] };
// Objects and groups in the arguments arrive by net id (DM-14).
_args = [_args] call skd_fnc_decode;
// The lead's step the order was sent from stamps the errors raised while it runs (ERR-01).
private _path = _order getOrDefault ["path", []];
if (isNil "_path") then { _path = [] };
skd_orderPaths pushBack [_id, _path];

skd_compileErrors = [];
skd_compiling = true;
private _code = compile (_order get "source");
skd_compiling = false;
if (skd_compileErrors isNotEqualTo []) exitWith {
    skd_orderPaths deleteAt (skd_orderPaths findIf { (_x select 0) isEqualTo _id });
    (skd_compileErrors select 0) params ["_text", "_line"];
    [createHashMapFromArray [
        ["kind", "result"], ["order", _id], ["state", "compile-error"],
        ["error", format ["%1 (line %2)", _text, _line]]
    ]] call skd_fnc_send;
};

private _start = diag_tickTime;
private _box = [nil];
private _script = [_args, _code, _box, _id] spawn {
    params ["_skd_args", "_skd_code", "_skd_box", "_skd_order"];
    // Named by its order, so inspect lists it and leaves out its own (INS-01).
    scriptName format ["skd_exec %1", _skd_order];
    private _skd_remote = true;
    // A switch turns the nothing of code ending in an assignment into nil (fn_step.sqf).
    _skd_box set [0, switch (true) do { default { _skd_args call _skd_code } }];
};
// A cancelled order (AGT-04) is terminated by the order of kind cancel, through skd_execs.
skd_execs set [_id, _script];
waitUntil { scriptDone _script || { diag_tickTime - _start > _ceiling } };
skd_execs deleteAt _id;

private _message = createHashMapFromArray [["kind", "result"], ["order", _id], ["duration_s", (diag_tickTime - _start) toFixed 3]];
if (scriptDone _script) then {
    _message set ["state", "done"];
    _message set ["value", [_box select 0] call skd_fnc_encode];
} else {
    terminate _script;
    _message set ["state", "ceiling"];
};
skd_orderPaths deleteAt (skd_orderPaths findIf { (_x select 0) isEqualTo _id });
[_message] call skd_fnc_send;
