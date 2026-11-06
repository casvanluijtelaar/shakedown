// @check US-014 CTL-04 code that does not compile on p1 ends fail check-error with the engine's compile message
// @tags US-014 CTL-04 DM-04 acceptance
// @setups ded

// compile of text that is not SQF gives code whose text is that text, and raises a script error on the
// lead machine, which this check expects. The code is sent to p1 as text and compiled there, where it
// does not compile.
["send broken code", {
    ["Invalid number in expression", {
        ["p1", compile "1 +"] call skd_fnc_on;
    }] call skd_fnc_expectError;
}] call skd_fnc_step;
["never runs", { true }] call skd_fnc_step;
