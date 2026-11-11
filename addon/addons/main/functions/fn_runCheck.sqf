/*
    Runs an order of kind check on the lead machine (P-05 steps 5 and 8): compiles the body,
    spawns it, waits for it to end or for its timeout, and sends the end report.

    Arguments: [order] - HashMap with "id", "check_id", "source", "timeout", and the session's
    "machines", "mode" and "setup" for the context functions (check API, "Context").
*/
params ["_order"];
private _timeout = _order getOrDefault ["timeout", 120];
skd_check = createHashMapFromArray [
    ["order", _order get "id"],
    ["id", _order get "check_id"],
    ["steps", []],
    ["stack", []],
    ["attached", []],
    ["expects", []],
    ["deadline", diag_tickTime + _timeout],
    ["machines", _order getOrDefault ["machines", [skd_machine]]],
    ["mode", _order getOrDefault ["mode", "singleplayer"]],
    ["setup", _order getOrDefault ["setup", createHashMap]],
    ["done", false]
];

skd_compileErrors = [];
skd_compiling = true;
private _body = compile (_order get "source");
skd_compiling = false;
if (skd_compileErrors isNotEqualTo []) then {
    (skd_compileErrors select 0) params ["_text", "_line"];
    skd_check set ["end", ["fail", "check-error",
        format ["The check body does not compile: %1, at line %2 of the check file. Fix the SQF and run the check again.", _text, _line],
        (call skd_fnc_clocks) get "utc", nil, nil]];
} else {
    private _script = [] spawn _body;
    skd_check set ["body", _script];
    // A run stopped during the check (RUN-05) terminates the body by the order of kind cancel,
    // through skd_execs; the runner no longer waits for this order's report.
    skd_execs set [_order get "id", _script];
    waitUntil { scriptDone _script || { diag_tickTime > (skd_check get "deadline") } };
    skd_execs deleteAt (_order get "id");
    if (!scriptDone _script) then {
        terminate _script;
        if (isNil { skd_check get "end" }) then {
            private _stack = skd_check get "stack";
            private _path = if (count _stack > 0) then {
                ((skd_check get "steps") select (_stack select -1)) get "path"
            } else { nil };
            private _where = if (isNil "_path") then { "" } else {
                format [" in step ""%1""", _path joinString " > "]
            };
            skd_check set ["end", ["fail", "check-timeout",
                format ["The check ran past its timeout of %1 s%2 while the game still answered. Raise @timeout in the check's header if the step needs longer, or find why it runs long.", _timeout, _where],
                (call skd_fnc_clocks) get "utc", nil, _path]];
        };
    };
};
skd_check set ["done", true];

private _end = skd_check getOrDefault ["end", []];
private _now = diag_tickTime;
private _steps = (skd_check get "steps") apply {
    private _step = createHashMapFromArray [
        ["path", _x get "path"],
        ["machine", skd_machine],
        ["state", _x get "state"],
        ["start", _x get "start"],
        ["duration_s", ((_x getOrDefault ["t1", _now]) - (_x get "t0")) toFixed 3]
    ];
    private _error = _x get "error";
    if (!isNil "_error") then { _step set ["error", _error] };
    private _stepEnd = _x get "end";
    if (!isNil "_stepEnd") then { _step set ["end", _stepEnd] };
    _step
};
private _evidence = createHashMapFromArray [
    ["attached", (skd_check get "attached") apply {
        createHashMapFromArray [["label", _x select 0], ["value", [_x select 1] call skd_fnc_encode]]
    }]
];
private _report = createHashMapFromArray [
    ["kind", "report"],
    ["order", skd_check get "order"],
    ["steps", _steps],
    ["evidence", _evidence],
    // Each skd_fnc_expectError block: its text, when it started and the step it was called in;
    // the runner matches them against the errors of every machine (ERR-04).
    ["expects", (skd_check get "expects") apply {
        createHashMapFromArray [["text", _x select 0], ["from", _x select 1], ["path", _x select 2]]
    }]
];
if (count _end == 0) then {
    _report set ["outcome", "pass"];
} else {
    _end params ["_outcome", "_kind", "_message", "_at", "_observed", "_path", ["_machine", skd_machine]];
    _report set ["outcome", _outcome];
    _report set ["reason", createHashMapFromArray [["kind", _kind], ["message", _message], ["at", _at], ["machine", _machine]]];
    if (!isNil "_observed") then { _evidence set ["observed", [_observed] call skd_fnc_encode] };
    if (!isNil "_path") then { _evidence set ["failed_step", _path] };
};
[_report] call skd_fnc_send;
