// @check DM-12 P-03 the groups all, clients and headless reach the machines they name, and code runs where it is sent
// @tags DM-12 P-03 CTL-01 CTL-02 DM-13 acceptance

private _kind = { if (isServer) then { "server" } else { if (hasInterface) then { "client" } else { "headless" } } };

["all reaches every machine, and each runs the code itself", {
    params ["_kind"];
    private _all = ["all", _kind] call skd_fnc_onEach;
    [count _all, "==", 4, "the machines that answered for all"] call skd_fnc_expect;
    [_all get "server", "==", "server", "what server is"] call skd_fnc_expect;
    [_all get "p1", "==", "client", "what p1 is"] call skd_fnc_expect;
    [_all get "p2", "==", "client", "what p2 is"] call skd_fnc_expect;
    [_all get "hc1", "==", "headless", "what hc1 is"] call skd_fnc_expect;
}, [_kind]] call skd_fnc_step;

["clients and headless name their machines", {
    params ["_kind"];
    private _clients = keys (["clients", _kind] call skd_fnc_onEach);
    _clients sort true;
    [_clients, "==", ["p1", "p2"], "the machines clients reaches"] call skd_fnc_expect;
    private _headless = ["headless", _kind] call skd_fnc_onEach;
    [keys _headless, "==", ["hc1"], "the machines headless reaches"] call skd_fnc_expect;
    private _two = keys ([["p2", "hc1"], _kind] call skd_fnc_onEach);
    _two sort true;
    [_two, "==", ["hc1", "p2"], "the machines an array of names reaches"] call skd_fnc_expect;
}, [_kind]] call skd_fnc_step;

["values and objects travel to a headless client and a client and back", {
    [["hc1", { (_this select 0) + 1 }, [41]] call skd_fnc_on, "==", 42, "hc1 added one to 41"] call skd_fnc_expect;
    private _p1 = ["p1"] call skd_fnc_player;
    [["hc1", { netId (_this select 0) }, [_p1]] call skd_fnc_on, "==", netId _p1, "the net id hc1 reads of p1's unit"] call skd_fnc_expect;
    [["p2", { _this select 0 }, [_p1]] call skd_fnc_on, "==", _p1, "p1's unit sent to p2 and back"] call skd_fnc_expect;
}] call skd_fnc_step;
