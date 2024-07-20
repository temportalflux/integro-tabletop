---
created: 2023-09-20T13:46:03.506Z
updated: 2023-09-21T15:31:52.753Z
assigned: ""
progress: 0
tags:
  - Optimization
---

# Expanded Object Cache

Additional Object Cache should save its bundles to the character kdl (and read them) so offline character sheets can be supported. Ideally we'd track what bundles are actually being used by the charcter so we don't reserialize bundles that aren't actively in use.
