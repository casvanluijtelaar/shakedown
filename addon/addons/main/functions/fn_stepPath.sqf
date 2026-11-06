/*
    The path of the innermost step running in the check on this machine, or [] outside a step.
*/
if (isNil "skd_check") exitWith { [] };
private _stack = skd_check get "stack";
if (count _stack == 0) exitWith { [] };
((skd_check get "steps") select (_stack select -1)) get "path"
