---
type: Finding
id: m05-1
milestone: M5
title: runner.json is readable by the Administrators group besides the current user
description: SEC-02, INT-WIN-03 and the command line contract promise a runner.json readable by the current user only; the file the runner writes inherits its folder's ACL, which also gives the Administrators group and SYSTEM full control.
raised_by: test
for: developer
checks: ["US-052 SEC-02 INT-WIN-03 while a runner runs, runner.json grants read to the current user and to no other account or group"]
stories: [US-052]
generated: { by: "claude-code/tester", at: 2026-11-09T03:43:44-08:00 }
addressed:
  - { by: "claude-code/developer", at: 2026-11-09T04:17:25-08:00, note: "The runner writes runner.json under a temporary name with a protected access list that grants full control to the current user alone, then renames it into place; the unit test sec_02_a_private_file_grants_the_current_user_only proves the list, and the named check passed 3 of 3. docs/sessions-and-the-runner.md, docs/install.md and docs/milestones/m00.md say so." }
verified:
  - { by: "claude-code/tester", at: 2026-11-09T04:22:07-08:00 }
---

# runner.json is readable by the Administrators group besides the current user

## What the spec promises

- `SEC-02` (must): "The runner token file is readable by the current user only."
- `INT-WIN-03`: "`%LOCALAPPDATA%\Shakedown\runner.json` (port, token, process id and start time, readable by the current user only, ...)".
- `spec/contract/cli.md`, "The runner's file and connection": "`%LOCALAPPDATA%\Shakedown\runner.json` is `{...}`, readable by the current user only (`INT-WIN-03`)."
- `US-052` lists `SEC-02`, and its story is that "only I can run code in my test session". The runner token in `runner.json` is what lets a client run code in the session (`SEC-01`).

## What the system does

With a runner running, started by `shakedown exec --machine p1 --code 1` with no session (exit 4), the access rules of `%LOCALAPPDATA%\Shakedown\runner.json`, read with `Get-Acl` and `GetAccessRules($true, $true, [SecurityIdentifier])`:

```
S-1-5-18      (SYSTEM)          Allow FullControl  inherited
S-1-5-32-544  (Administrators)  Allow FullControl  inherited
the current user                Allow FullControl  inherited
```

The rules are not protected (`AreAccessRulesProtected` is false), and the file has no rule of its own; all three come from the folder. Any account in the local Administrators group can therefore read the runner token. `docs/milestones/m00.md` says, under Left out, that the file "lives in the user's local application data folder, which only the user can read, and is written with no ACL of its own". On this machine that folder grants the same three principals, so the folder does not keep other administrators out either.

The check allows SYSTEM, which is the operating system itself and not an account a person or program logs on as. It fails only on the Administrators group. 3 of 3 runs failed the same way.

## What it costs

On a machine with more than one administrator account, another administrator's programs can read the token and drive the session, and so run code in the game as this user's test. The test machine has one user, so the cost there is small. An administrator can also take ownership of any file. Still, the promise is "the current user only", and both the contract and the M0 docs say it holds. Either the runner writes the file with a protected ACL that grants only the current user (and SYSTEM, if the spec allows it), or the spec, the contract and `docs/milestones/m00.md` say what the file's ACL is.

## The check that holds it

`US-052 SEC-02 INT-WIN-03 while a runner runs, runner.json grants read to the current user and to no other account or group`
