// @check US-008 The check after a failed one whose cleanup named p1 finds p1 damageable and the variable gone
// @tags US-008 RST-01 m10-2 acceptance

["p1 is damageable and skd_m10_2 is unset", {
    ["p1", { [isDamageAllowed player, missionNamespace getVariable ["skd_m10_2", 0]] }, "==", [true, 0],
        "p1's allowDamage and skd_m10_2 after the reset ran the cleanup", 10] call skd_fnc_expectOn;
}] call skd_fnc_step;
