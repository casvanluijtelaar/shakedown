// @check US-008 A cleanup left to the default machine runs on the server of a dedicated session and leaves p1's own state as the check set it
// @tags US-008 RST-01 m10-2 acceptance

// The control for m10-2: the reset itself does not put back allowDamage or a variable in p1's missionNamespace
// (docs/check-api.md, "The reset between checks"), and a cleanup that names no machine runs on the lead, the server.
["p1", { player allowDamage false; missionNamespace setVariable ["skd_m10_2", 1]; true }] call skd_fnc_on;
[{ if (!isNull player) then { player allowDamage true }; missionNamespace setVariable ["skd_m10_2", nil] }] call skd_fnc_cleanup;
["p1", { [isDamageAllowed player, missionNamespace getVariable ["skd_m10_2", 0]] }, "==", [false, 1],
    "p1's allowDamage and skd_m10_2 as the check set them", 10] call skd_fnc_expectOn;
