// @check US-037 A check with nested steps, a value from p1 and evidence passes
// @tags US-037 smoke

// The value comes from code sent to p1, which sees only what it is given in args.
private _answer = ["ask p1 for a value", {
    ["p1", { (_this select 0) + 1 }, [1]] call skd_fnc_on
}] call skd_fnc_step;

["the answer is two", {
    ["compare it", {
        [_this select 0, "==", 2, "1 + 1 on p1"] call skd_fnc_expect;
    }, _this] call skd_fnc_step;
}, [_answer]] call skd_fnc_step;

["answer", _answer] call skd_fnc_evidence;
