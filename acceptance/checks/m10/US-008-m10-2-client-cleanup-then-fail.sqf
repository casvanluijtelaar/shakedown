// @check US-008 A cleanup that names p1 undoes p1's allowDamage and a variable in its missionNamespace though the check fails first
// @tags US-008 RST-01 m10-2 acceptance

// docs/check-api.md, "Cleanups and ending a check": state local to one client is undone on that client, so its
// cleanup names it. The check sets two such pieces of state on p1, registers a cleanup for p1, and fails before
// it puts anything back itself.
["p1 makes its player unhurtable and sets a variable", {
    ["p1", { player allowDamage false; missionNamespace setVariable ["skd_m10_2", 1]; true }] call skd_fnc_on;
    [{ player allowDamage true; missionNamespace setVariable ["skd_m10_2", nil] }, [], "p1"] call skd_fnc_cleanup;
    ["p1", { [isDamageAllowed player, missionNamespace getVariable ["skd_m10_2", 0]] }, "==", [false, 1],
        "p1's allowDamage and skd_m10_2 as the check set them", 10] call skd_fnc_expectOn;
}] call skd_fnc_step;

["fail before putting anything back", {
    ["the check fails on purpose before it puts p1's state back"] call skd_fnc_fail;
}] call skd_fnc_step;
