---
type: Index
title: M5 findings
description: The findings of M5 A session that outlives its client.
---

# M5 findings

* [runner.json is readable by the Administrators group besides the current user](01-runner-json-is-readable-by-the-administrators-group-besides-the-current-user.md) - SEC-02, INT-WIN-03 and the command line contract promise a runner.json readable by the current user only; the file the runner writes inherits its folder's ACL, which also gives the Administrators group and SYSTEM full control.
