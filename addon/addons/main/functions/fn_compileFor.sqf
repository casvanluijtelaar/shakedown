/*
    Compiles code that a check sends to a machine from its text, so it does not see the check's
    local variables (CTL-02). A compile error ends the check fail / check-error with the engine's
    message (CTL-04).

    Arguments: [function, machine, code]
    Returns: the compiled code, or nothing when the check has ended.
*/
params ["_function", "_machine", "_code"];
skd_compileErrors = [];
skd_compiling = true;
private _compiled = compile toString _code;
skd_compiling = false;
if (skd_compileErrors isNotEqualTo []) exitWith {
    (skd_compileErrors select 0) params ["_text", "_line"];
    ["fail", "check-error", format ["%1: the code does not compile on %2: %3 (line %4 of the code).", _function, _machine, _text, _line]] call skd_fnc_end;
    nil
};
_compiled
