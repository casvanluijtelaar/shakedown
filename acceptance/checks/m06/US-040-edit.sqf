// @check US-040 a check edited between two runs runs its new text
// @tags US-040 RLD-04 API-02 NFR-PF-01 acceptance

["the value the check expects", {
    [1, "==", 1, "acc_m06 the edited expectation"] call skd_fnc_expect;
}] call skd_fnc_step;
