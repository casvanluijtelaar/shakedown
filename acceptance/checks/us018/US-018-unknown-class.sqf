// @check US-018 An unknown class name given to skd_fnc_create is the author's to fix
// @tags US-018 acceptance

["build the scene", {
    ["create a soldier with a typo in its class", {
        ["B_Soldier_Typo", AGLToASL [110, 110, 0]] call skd_fnc_create;
    }] call skd_fnc_step;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
