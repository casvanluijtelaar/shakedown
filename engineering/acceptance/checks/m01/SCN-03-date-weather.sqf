// @check SCN-03 skd_fnc_setDate and skd_fnc_setWeather change the date, time and weather every machine reports
// @tags SCN-03 acceptance

["set the date", {
    private _done = [[2035, 6, 24, 22, 15]] call skd_fnc_setDate;
    [_done, "==", true, "the value skd_fnc_setDate returned"] call skd_fnc_expect;
    ["all", { date select [0, 5] }, "==", [2035, 6, 24, 22, 15], "every machine's date to the minute", 2] call skd_fnc_expectEach;
}] call skd_fnc_step;

["set the weather", {
    private _done = [0.9, 0.2, 0.6] call skd_fnc_setWeather;
    [_done, "==", true, "the value skd_fnc_setWeather returned"] call skd_fnc_expect;
    ["all", { overcast }, "~=", [0.9, 0.01], "every machine's overcast", 2] call skd_fnc_expectEach;
}] call skd_fnc_step;
