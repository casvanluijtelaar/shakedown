// @check US-094 a check of 25 s that would pass, during which the harness starves p2 of CPU
["p2", { diag_frameNo }] call skd_fnc_on;
["p1", { false }, "25 s to pass", 25] call skd_fnc_tryWaitUntil;
