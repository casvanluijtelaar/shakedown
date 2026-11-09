// The fixture mod @skd_fixture_cba (spec/contract/fixture-mod.md). It needs CBA_A3 and
// @skd_fixture.
class CfgPatches {
    class skdfx_cba {
        name = "Shakedown fixture with CBA";
        author = "casvanluijtelaar";
        units[] = {};
        weapons[] = {};
        requiredVersion = 2.18;
        requiredAddons[] = {"cba_main", "skdfx_main"};
        version = 0.0;
        versionStr = "0.0.4";
        versionAr[] = {0, 0, 4};
    };
};

// The reload target behind a mission restart (contract "A reload target behind a mission
// restart"): skdfx_cba_fnc_version is a CBA PREP function, compiled as CBA mods do in XEH
// preStart and again in preInit, so it is final and changes only at a mission start with CBA's
// cache off.
class Extended_PreStart_EventHandlers {
    class skdfx_cba {
        init = "call compile preprocessFileLineNumbers '\skdfx_cba\addons\main\XEH_preStart.sqf'";
    };
};

class Extended_PreInit_EventHandlers {
    class skdfx_cba {
        init = "call compile preprocessFileLineNumbers '\skdfx_cba\addons\main\XEH_preInit.sqf'";
    };
};

class Extended_PostInit_EventHandlers {
    class skdfx_cba {
        clientInit = "call compile preprocessFileLineNumbers '\skdfx_cba\addons\main\XEH_postInitClient.sqf'";
    };
};
