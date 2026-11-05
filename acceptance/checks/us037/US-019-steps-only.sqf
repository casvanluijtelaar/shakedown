// @check US-019 A check made of nothing but a header and steps passes

["add two numbers", {
    1 + 1
}] call skd_fnc_step;

["join two strings", {
    "steps " + "only"
}] call skd_fnc_step;
