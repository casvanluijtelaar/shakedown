/*
    The wait of `shakedown wait` and the wait_until tool (OBS-01): evaluates the condition every
    frame until it returns true or the ceiling passes. A condition that raises a script error or
    returns anything but a Boolean ends the wait at that evaluation, with the cause.

    Arguments: [condition source, args, ceiling]
    Returns: HashMap with "met", "waited_s" and, when the wait ended early, "error"; or "compile"
    with the engine's message when the condition does not compile.
*/
params ["_skd_source", ["_skd_args", []], ["_skd_ceiling", 10]];
skd_compileErrors = [];
skd_compiling = true;
private _skd_code = compile _skd_source;
skd_compiling = false;
if (skd_compileErrors isNotEqualTo []) exitWith {
    (skd_compileErrors select 0) params ["_text", "_line"];
    createHashMapFromArray [["compile", format ["%1 (line %2)", _text, _line]]]
};
private _skd_start = diag_tickTime;
private _skd_met = false;
private _skd_error = "";
while { !_skd_met && { _skd_error isEqualTo "" } && { diag_tickTime - _skd_start <= _skd_ceiling } } do {
    private _skd_errors = skd_errorCount;
    private _skd_value = switch (true) do { default { _skd_args call _skd_code } };
    if (skd_errorCount > _skd_errors) then {
        _skd_error = format ["the condition raised a script error: %1", skd_lastError];
    } else {
        if (isNil "_skd_value") then {
            _skd_error = "the condition returned nothing, not a Boolean";
        } else {
            if (_skd_value isEqualType true) then {
                _skd_met = _skd_value;
            } else {
                _skd_error = format ["the condition returned %1 (%2), not a Boolean", [[_skd_value]] call skd_fnc_typeName, str _skd_value];
            };
        };
    };
    if (!_skd_met && { _skd_error isEqualTo "" }) then {
        private _skd_frame = diag_frameNo;
        waitUntil { diag_frameNo > _skd_frame };
    };
};
private _skd_out = createHashMapFromArray [["met", _skd_met], ["waited_s", (diag_tickTime - _skd_start) toFixed 3]];
if (_skd_error isNotEqualTo "") then { _skd_out set ["error", _skd_error] };
_skd_out
