class CfgPatches {
    class shakedown_main {
        name = "Shakedown";
        author = "casvanluijtelaar";
        units[] = {};
        weapons[] = {};
        requiredVersion = 2.18;
        requiredAddons[] = {"A3_Functions_F"};
        version = 0.0;
        versionStr = "0.0.1";
        versionAr[] = {0, 0, 1};
    };
};

class CfgFunctions {
    class skd {
        // The in-game agent: receives orders from the extension, runs them, sends results back.
        class agent {
            file = "\z\shakedown\addons\main\functions";
            class init { postInit = 1; };
            class receive {};
            class send {};
            class clocks {};
            class encode {};
            class runExec {};
            class runCheck {};
            class end {};
            class argError {};
            class guard {};
        };
        // The check API (spec/contract/check-api.md).
        class api {
            file = "\z\shakedown\addons\main\functions";
            class step {};
            class on {};
            class expect {};
            class evidence {};
            class fail {};
        };
    };
};
