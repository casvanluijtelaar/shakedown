// @check US-008 After a check whose cleanup ran on the server, p1 is still unhurtable with the variable set; this check puts both back on p1
// @tags US-008 RST-01 m10-2 acceptance

[{ player allowDamage true; missionNamespace setVariable ["skd_m10_2", nil] }, [], "p1"] call skd_fnc_cleanup;
["p1 kept the state the earlier check set", {
    ["p1", { [isDamageAllowed player, missionNamespace getVariable ["skd_m10_2", 0]] }, "==", [false, 1],
        "p1's allowDamage and skd_m10_2 after a reset whose cleanup ran on the server", 10] call skd_fnc_expectOn;
}] call skd_fnc_step;
