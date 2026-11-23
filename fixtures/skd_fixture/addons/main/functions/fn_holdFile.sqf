/*
    skdfx_fnc_holdFile [path] - the fixture's extension opens the file at `path`, relative to the
    game folder, for reading, shared for reading only, and keeps it open until the game process
    ends, as the game holds a loose file it read (FIX-04, engineering/spec/contract/fixture-mod.md). Returns
    true when the file was opened, false when it does not exist, cannot be opened or the extension
    is not loaded.
*/
params [["_path", "", [""]]];
if (_path isEqualTo "") exitWith { false };
private _answer = "skdfx" callExtension ["hold", [_path]];
(_answer select 0) isEqualTo "true"
