// @check US-064 DM-11 name player reads p1 on p1 and p2 on p2
// @tags US-064 DM-11 NFR-PV-02 acceptance

["each client's player has its machine name", {
    ["p1", { name player }, "==", "p1", "p1's player's name"] call skd_fnc_expectOn;
    ["p2", { name player }, "==", "p2", "p2's player's name"] call skd_fnc_expectOn;
}] call skd_fnc_step;
