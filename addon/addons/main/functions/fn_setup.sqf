/*
    skd_fnc_setup [] - the session's setup as a HashMap with "name", "mode", "clients",
    "headless" and "world" (check API, "Context").
*/
if !(["skd_fnc_setup"] call skd_fnc_guard) exitWith {};
+(skd_check get "setup")
