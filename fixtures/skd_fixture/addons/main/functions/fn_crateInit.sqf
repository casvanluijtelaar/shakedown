// The crate's init: on every machine with an interface, the action "Open crate", radius 3 m,
// shown while the crate is not open. Firing it asks the server to open the crate.
params ["_crate"];
if (!hasInterface) exitWith {};
_crate addAction [
    "Open crate",
    { params ["_target", "_caller"]; [_target, _caller] remoteExec ["skdfx_fnc_open", 2] },
    nil, 1.5, true, true, "",
    "!(_target getVariable ['skdfx_open', false])",
    3
];
