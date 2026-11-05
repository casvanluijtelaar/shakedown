// The fixture mod @skd_fixture (spec/contract/fixture-mod.md). It needs nothing but the game. Each
// behaviour of the contract is added with the first story or demo that uses it; a setup without
// CBA loads this mod alone.
class CfgPatches {
    class skdfx_main {
        name = "Shakedown fixture";
        author = "casvanluijtelaar";
        units[] = {};
        weapons[] = {};
        requiredVersion = 2.18;
        requiredAddons[] = {"A3_Functions_F"};
        version = 0.0;
        versionStr = "0.0.2";
        versionAr[] = {0, 0, 2};
    };
};
