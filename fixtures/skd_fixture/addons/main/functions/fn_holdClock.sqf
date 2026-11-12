/*
    skdfx_fnc_holdClock [seconds] - in singleplayer, holds p1's mission time still for `seconds`
    of real time (1 to 30) while its frames go on and it keeps answering, then lets it run (FIX-02,
    spec/contract/fixture-mod.md). Returns true at once; on any other machine, or in a dedicated
    session, it does nothing and returns false.
*/
params [["_seconds", 0, [0]]];
if (isMultiplayer || { !hasInterface } || { _seconds < 1 } || { _seconds > 30 }) exitWith { false };
skdfx_holdUntil = diag_tickTime + _seconds;
setAccTime 0;
if (isNil "skdfx_holdHandler") then {
    skdfx_holdHandler = addMissionEventHandler ["EachFrame", {
        if (diag_tickTime >= skdfx_holdUntil) then {
            setAccTime 1;
            removeMissionEventHandler ["EachFrame", _thisEventHandler];
            skdfx_holdHandler = nil;
        };
    }];
};
true
