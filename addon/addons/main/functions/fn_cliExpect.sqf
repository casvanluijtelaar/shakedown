/*
    The assertion of `shakedown expect` and the expect tool (OBS-06): reads the value every frame
    until the comparison with the expected value holds or the ceiling passes, as skd_fnc_expectOn
    does in a check.

    Arguments: [value source, op, expected, args, ceiling]
    Returns: HashMap with "passed", "last_value" (absent for nothing), "waited_s", and
    "type_error", what the operator needs, when the operands do not fit it; or "compile" with the
    engine's message when the value's code does not compile.
*/
params ["_skd_source", "_skd_op", "_skd_expected", ["_skd_args", []], ["_skd_ceiling", 10]];
skd_compileErrors = [];
skd_compiling = true;
private _skd_code = compile _skd_source;
skd_compiling = false;
if (skd_compileErrors isNotEqualTo []) exitWith {
    (skd_compileErrors select 0) params ["_text", "_line"];
    createHashMapFromArray [["compile", format ["%1 (line %2)", _text, _line]]]
};
private _skd_expectedW = if (isNil "_skd_expected") then { [] } else { [_skd_expected] };
private _skd_start = diag_tickTime;
private _skd_held = false;
private _skd_typeError = "";
private _skd_last = [];
while { !_skd_held && { _skd_typeError isEqualTo "" } && { diag_tickTime - _skd_start <= _skd_ceiling } } do {
    private _skd_value = switch (true) do { default { _skd_args call _skd_code } };
    _skd_last = if (isNil "_skd_value") then { [] } else { [_skd_value] };
    private _skd_r = [_skd_last, _skd_op, _skd_expectedW] call skd_fnc_compare;
    _skd_held = _skd_r select 0;
    _skd_typeError = _skd_r select 1;
    if (!_skd_held && { _skd_typeError isEqualTo "" }) then {
        private _skd_frame = diag_frameNo;
        waitUntil { diag_frameNo > _skd_frame };
    };
};
private _skd_out = createHashMapFromArray [["passed", _skd_held], ["waited_s", (diag_tickTime - _skd_start) toFixed 3]];
if (_skd_last isNotEqualTo []) then { _skd_out set ["last_value", _skd_last select 0] };
if (_skd_typeError isNotEqualTo "") then { _skd_out set ["type_error", _skd_typeError] };
_skd_out
