---
type: Index
title: Contract
description: The framework's public interface, which the test developer works from and never past - the check API, the command line, the MCP tools and the result file with its JSON Schema, and the fixture mods' behaviour.
---

# Contract

* [Check API](check-api.md) - The check file and its header, and every skd_fnc function a check may call.
* [Command line](cli.md) - The shakedown command, the shakedown.toml project file, the JSON output and the exit codes.
* [MCP tools](mcp-tools.md) - The MCP server's tools with their inputs, outputs and errors.
* [Result file](result-file.md) - The run folder and what each field of the result file means.
* [Fixture mods](fixture-mod.md) - The two small mods the framework's tests and the acceptance suite run against, and what each one promises.
* [Result file schema](result.schema.json) - The JSON Schema (2020-12) every result file validates against.
