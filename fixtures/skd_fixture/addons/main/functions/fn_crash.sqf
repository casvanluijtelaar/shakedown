/*
    skdfx_fnc_crash [delay] - after `delay` seconds of real time (0 to 60), the game process this
    runs on faults in native code inside skdfx_x64.dll, an access violation, and ends unasked with
    the engine's own crash handling (FIX-01, spec/contract/fixture-mod.md). Returns true at once,
    false for a delay out of range or when the extension is not loaded.
*/
params [["_delay", 0, [0]]];
if (_delay < 0 || { _delay > 60 }) exitWith { false };
private _answer = "skdfx" callExtension ["crash", [_delay]];
(_answer select 0) isEqualTo "true"
