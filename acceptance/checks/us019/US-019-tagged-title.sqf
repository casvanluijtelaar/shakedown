// @check US-019 ACT-01 an action whose title is structured text is found by its title with the tags removed
// @tags US-019 ACT-01 acceptance

private _box = ["put a box with a tagged action 2 m in front of p1", {
    ["p1", AGLToASL [210, 200, 0], 0] call skd_fnc_place;
    private _box = ["Land_WoodenBox_F", AGLToASL [210, 202, 0], 0] call skd_fnc_create;
    ["p1", { !isNull objectFromNetId (_this select 0) }, "the box reached p1", 10, [netId _box]] call skd_fnc_waitUntil;
    ["p1", {
        (objectFromNetId (_this select 0)) addAction [
            "<t color='#ff8800'>Mark <t size='1.2'>the box</t></t>",
            { (_this select 0) setVariable ["acc_m02_marked", true, true] }
        ];
        true
    }, [netId _box]] call skd_fnc_on;
    _box
}] call skd_fnc_step;

["p1 uses the action by its plain title", {
    params ["_box"];
    private _used = ["p1", _box, "Mark the box"] call skd_fnc_useAction;
    [_used, "==", true, "the value skd_fnc_useAction returned"] call skd_fnc_expect;
    ["server", { (_this select 0) getVariable ["acc_m02_marked", false] }, "==", true, "the box is marked", 10, [_box]] call skd_fnc_expectOn;
}, [_box]] call skd_fnc_step;
