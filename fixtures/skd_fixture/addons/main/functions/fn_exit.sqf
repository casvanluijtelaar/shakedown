/*
    skdfx_fnc_exit [delay, code] - after `delay` seconds of real time (0 to 60), the game process
    this runs on ends with exit code `code` (1 to 2147483647), leaving no dump (FIX-01,
    spec/contract/fixture-mod.md). Returns true at once, false for an argument out of range or
    when the extension is not loaded.
*/
params [["_delay", 0, [0]], ["_code", 1, [0]]];
if (_delay < 0 || { _delay > 60 } || { _code < 1 } || { _code > 2147483647 } || { _code != floor _code }) exitWith { false };
// toFixed keeps every digit of a large code, which str would round.
private _answer = "skdfx" callExtension ["exit", [_delay, _code toFixed 0]];
(_answer select 0) isEqualTo "true"
