/*
    Shows the list of a hand over (MAN-01) on this machine's screen, at its top left under the
    title "Shakedown: look for", numbered, until the next reset removes it, and writes it to the RPT.

    Arguments: [items] - Array of Strings.
    Returns: true once the list is on the screen, false when this machine has no game window.
*/
params ["_items"];
disableSerialization;
diag_log text format ["[shakedown] handed over to %1; look for: %2", skd_machine, _items joinString " | "];
if (!hasInterface) exitWith { false };
waitUntil { !isNull findDisplay 46 };
call skd_fnc_handOverClear;
// Structured text reads <, > and & as markup, so the items are escaped.
private _escape = {
    params ["_text"];
    ((_text regexReplace ["&", "&amp;"]) regexReplace ["<", "&lt;"]) regexReplace [">", "&gt;"]
};
private _lines = ["<t size='1.15' color='#ffd24d'>Shakedown: look for</t>"];
{ _lines pushBack format ["%1. %2", _forEachIndex + 1, [_x] call _escape] } forEach _items;
private _ctrl = (findDisplay 46) ctrlCreate ["RscStructuredText", -1];
private _w = 0.34 * safeZoneW;
_ctrl ctrlSetPosition [safeZoneX + 0.01 * safeZoneW, safeZoneY + 0.12 * safeZoneH, _w, 0.5 * safeZoneH];
_ctrl ctrlSetBackgroundColor [0, 0, 0, 0.65];
_ctrl ctrlSetStructuredText parseText (_lines joinString "<br/>");
_ctrl ctrlCommit 0;
_ctrl ctrlSetPositionH (ctrlTextHeight _ctrl + 0.01);
_ctrl ctrlCommit 0;
uiNamespace setVariable ["skd_handOver", _ctrl];
!isNull _ctrl
