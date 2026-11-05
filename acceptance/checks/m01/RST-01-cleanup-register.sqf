// @check RST-01 A check registers two cleanups for the next reset
// @tags RST-01 acceptance

acc_m01_cleanups = [];
[{ acc_m01_cleanups pushBack (_this select 0) }, [1]] call skd_fnc_cleanup;
[{ acc_m01_cleanups pushBack (_this select 0) }, [2]] call skd_fnc_cleanup;

["the cleanups have not run yet", {
    [acc_m01_cleanups, "==", [], "the cleanups that ran"] call skd_fnc_expect;
}] call skd_fnc_step;
