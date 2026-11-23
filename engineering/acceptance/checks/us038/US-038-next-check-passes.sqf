// @check US-038 The check after a broken one runs and passes
// @tags US-038 acceptance

private _answer = ["ask p1 for a sum", {
    ["p1", { (_this select 0) + (_this select 1) }, [2, 3]] call skd_fnc_on
}] call skd_fnc_step;

["the sum is five", {
    [_this select 0, "==", 5, "2 + 3 computed on p1"] call skd_fnc_expect;
}, [_answer]] call skd_fnc_step;
