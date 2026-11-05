/*
    skd_fnc_machines [] - the session's machine names, in the order server, p1..pN, hc1..hcN
    (check API, "Context"). In singleplayer ["p1"].
*/
if !(["skd_fnc_machines"] call skd_fnc_guard) exitWith {};
+(skd_check get "machines")
