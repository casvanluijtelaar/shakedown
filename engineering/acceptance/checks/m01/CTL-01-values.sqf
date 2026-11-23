// @check CTL-01 Values cross to a machine and back as DM-13 and DM-14 say
// @tags CTL-01 CTL-02 CTL-03 DM-13 DM-14 acceptance

private _box = ["create a box", {
    ["Land_WoodenBox_F", AGLToASL [114, 114, 0]] call skd_fnc_create
}] call skd_fnc_step;
private _group = group (["p1"] call skd_fnc_player);

["plain values come back as they went", {
    private _value = [1.5, "text", true, [1, [2, "three"]]];
    private _back = ["p1", { _this }, _value] call skd_fnc_on;
    [_back, "==", _value, "numbers, a string, a boolean and nested arrays"] call skd_fnc_expect;
}] call skd_fnc_step;

["an object and a group arrive as themselves", {
    private _seen = ["p1", {
        params ["_obj", "_grp"];
        [typeName _obj, typeOf _obj, typeName _grp, _grp == group player]
    }, [_this select 0, _this select 1]] call skd_fnc_on;
    [_seen, "==", ["OBJECT", "Land_WoodenBox_F", "GROUP", true], "the type and class of the object and group p1 received"] call skd_fnc_expect;
}, [_box, _group]] call skd_fnc_step;

["an object and a group come back as themselves", {
    private _back = ["p1", { [_this select 0, _this select 1] }, [_this select 0, _this select 1]] call skd_fnc_on;
    [_back select 0, "==", _this select 0, "the object returned by p1"] call skd_fnc_expect;
    [_back select 1, "==", _this select 1, "the group returned by p1"] call skd_fnc_expect;
}, [_box, _group]] call skd_fnc_step;

["a hashmap with string keys comes back as a hashmap", {
    private _back = ["p1", { createHashMapFromArray [["a", 1], ["b", "two"]] }] call skd_fnc_on;
    [typeName _back, "==", "HASHMAP", "the type of the returned hashmap"] call skd_fnc_expect;
    [_back get "a", "==", 1, "key a"] call skd_fnc_expect;
    [_back get "b", "==", "two", "key b"] call skd_fnc_expect;
}] call skd_fnc_step;

["code in args does not arrive as code", {
    private _seen = ["p1", { typeName (_this select 0) }, [{ 1 + 1 }]] call skd_fnc_on;
    [_seen, "!=", "CODE", "the type p1 received for a code argument"] call skd_fnc_expect;
}] call skd_fnc_step;

["the code does not see the caller's local variables", {
    private _secret = 5;
    private _seen = ["p1", { isNil "_secret" }] call skd_fnc_on;
    [_seen, "==", true, "isNil _secret on p1"] call skd_fnc_expect;
}] call skd_fnc_step;

["the code runs scheduled and may wait", {
    private _seen = ["p1", {
        private _until = diag_tickTime + 0.2;
        waitUntil { diag_tickTime >= _until };
        canSuspend
    }] call skd_fnc_on;
    [_seen, "==", true, "canSuspend after a waitUntil on p1"] call skd_fnc_expect;
}] call skd_fnc_step;
