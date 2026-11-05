// @check US-018 The check after a wrong argument runs and passes
// @tags US-018 acceptance

private _box = ["create a box", {
    ["Land_WoodenBox_F", AGLToASL [112, 112, 0]] call skd_fnc_create
}] call skd_fnc_step;

["the box exists", {
    [typeOf (_this select 0), "==", "Land_WoodenBox_F", "the class of the created box"] call skd_fnc_expect;
}, [_box]] call skd_fnc_step;
