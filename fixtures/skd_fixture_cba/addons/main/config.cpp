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
        versionStr = "0.0.2";
        versionAr[] = {0, 0, 2};
    };
};

class Extended_PostInit_EventHandlers {
    class skdfx_cba {
        clientInit = "call compile preprocessFileLineNumbers '\skdfx_cba\addons\main\XEH_postInitClient.sqf'";
    };
};
