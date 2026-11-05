// @check US-018 A string where skd_fnc_place takes a position is the author's to fix
// @tags US-018 acceptance

["move p1", {
    ["p1", "here", 0] call skd_fnc_place;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
