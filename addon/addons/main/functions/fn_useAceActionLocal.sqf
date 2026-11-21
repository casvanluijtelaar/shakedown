/*
    The part of skd_fnc_useAceAction that runs on the acting machine (ACT-04, INT-ACE-03): waits,
    every frame to the ceiling, until every name of the path is found in the target's ACE menu and
    every action on it has a condition that holds, then selects the last action as ACE's menu does
    and calls ACE's key-up code, which checks its condition again and runs its statement (SPK-21).

    The path is walked as ACE's menu builds it (ace_interact_menu_fnc_collectActiveActionTree):
    at each level the action's class children, the actions added to that one object whose parent
    path is the path so far, and the children its insert-children code makes. ACE's conditions and
    statements read _target, _player and _actionParams from the caller, so they are set here.

    Arguments: [target net id, path, ceiling]
    Returns: ["fired"], ["no-player"], ["no-ace"], ["ace-changed", version, internal], or the last
    state: ["missing", path found so far, names at that level], ["condition", path to the action
    whose condition was false], ["folder", its actions] or ["no-target"].
*/
params ["_netId", "_path", "_ceiling"];
if (!hasInterface || { isNull player }) exitWith { ["no-player"] };
if !(isClass (configFile >> "CfgPatches" >> "ace_interact_menu")) exitWith { ["no-ace"] };
private _internals = ["ace_interact_menu_fnc_keyUp", "ace_interact_menu_fnc_compileMenu", "ace_interact_menu_fnc_compileMenuSelfAction", "ace_interact_menu_ActNamespace", "ace_interact_menu_ActSelfNamespace", "ace_interact_menu_openedMenuType"];
private _lacks = _internals findIf { isNil _x };
if (_lacks >= 0) exitWith { ["ace-changed", getText (configFile >> "CfgPatches" >> "ace_main" >> "versionStr"), _internals select _lacks] };

private _fnc_try = {
    private _obj = objectFromNetId _netId;
    if (isNull _obj) exitWith { ["no-target"] };
    private _type = parseNumber (_obj isEqualTo player);
    private _result = [];
    isNil {
        private _player = ACE_player;
        private _target = _obj;
        private _actionParams = [];
        if (_type == 0) then { [_obj] call ace_interact_menu_fnc_compileMenu } else { [_obj] call ace_interact_menu_fnc_compileMenuSelfAction };
        private _objActions = _obj getVariable [["ace_interact_menu_actions", "ace_interact_menu_selfActions"] select _type, []];
        private _ns = [ace_interact_menu_ActNamespace, ace_interact_menu_ActSelfNamespace] select _type;
        private _level = (_ns getOrDefault [typeOf _obj, []]) apply { [_x select 0, _x select 1, _obj] };
        { if ((_x select 1) isEqualTo []) then { _level pushBack [_x select 0, [], _obj] } } forEach _objActions;
        private _full = [];
        private _node = [];
        {
            private _name = _x;
            private _i = _level findIf { ((_x select 0) select 0) isEqualTo _name };
            if (_i < 0) exitWith { _result = ["missing", +_full, _level apply { (_x select 0) select 0 }] };
            (_level select _i) params ["_data", "_children", "_on"];
            _target = _on;
            _actionParams = _data select 6;
            if ((_data select 10) isNotEqualTo {}) then { _data = +_data; [_target, _player, _actionParams, _data] call (_data select 10) };
            private _ok = [_target, _player, _actionParams] call (_data select 4);
            if (isNil "_ok" || { !(_ok isEqualTo true) }) exitWith { _result = ["condition", _full + [_name]] };
            _full pushBack _name;
            _node = [_data, _children, _on];
            _level = [];
            if ((_data select 5) isNotEqualTo {}) then {
                private _dynamic = [_target, _player, _actionParams] call (_data select 5);
                if (!isNil "_dynamic" && { _dynamic isEqualType [] }) then { { _level pushBack _x } forEach _dynamic };
            };
            { _level pushBack [_x select 0, _x select 1, _on] } forEach _children;
            { if ((_x select 1) isEqualTo _full) then { _level pushBack [_x select 0, [], _on] } } forEach _objActions;
        } forEach _path;
        if (_result isEqualTo [] && { ((_node select 0) select 3) isEqualTo {} }) then {
            _result = ["folder", _level apply { (_x select 0) select 0 }];
        };
        if (_result isEqualTo []) then {
            // As ACE's menu leaves it when the player releases the key on this action (fnc_render).
            ace_interact_menu_openedMenuType = _type;
            ace_interact_menu_actionSelected = true;
            ace_interact_menu_selectedTarget = _node select 2;
            ace_interact_menu_selectedAction = _node;
            [_type, true] call ace_interact_menu_fnc_keyUp;
            _result = ["fired"];
        };
    };
    _result
};

private _start = diag_tickTime;
private _state = [];
waitUntil {
    _state = call _fnc_try;
    (_state select 0) isEqualTo "fired" || { diag_tickTime - _start > _ceiling }
};
_state
