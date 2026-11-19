/*
    Shows the list of a hand over (MAN-01) on this machine's screen, at its top centre under the
    title "Shakedown: look for", numbered, until the next reset removes it, and writes it to the RPT.

    The box sits in the column that the game's default HUD layout leaves free at the top centre:
    right of the vehicle info and left of the weapon info. So, whatever its height, it covers
    neither the action menu and the communication menu (left column) nor the weapon info, stance,
    stamina bar and hints (right column). The bounds are those of the layout macros in the game's
    a3\ui_f\hpp\defineCommonGrids.inc, in units of its GUI grid, which hold at every aspect ratio
    and interface size.

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
// GUI_GRID_W and GUI_GRID_H of the game's layout macros.
private _gw = ((safeZoneW / safeZoneH) min 1.2) / 40;
private _gh = (((safeZoneW / safeZoneH) min 1.2) / 1.2) / 25;
// The vehicle info ends 10.5 grid units right of the safe zone's left edge and the weapon info
// starts 14.3 left of its right edge; one unit of space on each side, at most 24 units wide.
private _left = safeZoneX + 11.5 * _gw;
private _right = safeZoneX + safeZoneW - 15.3 * _gw;
private _w = (_right - _left) min (24 * _gw);
_ctrl ctrlSetPosition [(_left + _right - _w) / 2, safeZoneY + 0.5 * _gh, _w, 0.5 * safeZoneH];
_ctrl ctrlSetBackgroundColor [0, 0, 0, 0.65];
_ctrl ctrlSetStructuredText parseText (_lines joinString "<br/>");
_ctrl ctrlCommit 0;
_ctrl ctrlSetPositionH (ctrlTextHeight _ctrl + 0.01);
_ctrl ctrlCommit 0;
uiNamespace setVariable ["skd_handOver", _ctrl];
!isNull _ctrl
