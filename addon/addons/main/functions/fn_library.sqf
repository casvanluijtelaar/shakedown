/*
    Compiles the project's library on this machine (API-06): each file's code goes into the
    global variable skd_lib_<file name>, and the variables of the library this machine compiled
    before are cleared first, so a file deleted since leaves none behind. Called by the runner
    after each mission start and each reload.

    Arguments: [files] - each [name, path as shown, source with its comments turned into spaces].
    Returns: an array of [path, line, message] for each file that does not compile.
*/
params ["_files"];
{ missionNamespace setVariable [_x, nil] } forEach (missionNamespace getVariable ["skd_libNames", []]);
skd_libNames = [];
private _errors = [];
{
    _x params ["_name", "_path", "_source"];
    private _var = "skd_lib_" + _name;
    skd_compileErrors = [];
    skd_compiling = true;
    private _code = compile _source;
    skd_compiling = false;
    if (skd_compileErrors isNotEqualTo []) then {
        (skd_compileErrors select 0) params ["_text", "_line"];
        _errors pushBack [_path, _line, _text];
    } else {
        missionNamespace setVariable [_var, _code];
        skd_libNames pushBack _var;
    };
} forEach _files;
_errors
