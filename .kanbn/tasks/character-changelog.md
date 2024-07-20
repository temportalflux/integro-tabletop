---
created: 2023-09-20T13:51:39.013Z
updated: 2023-09-20T13:51:39.009Z
assigned: ""
progress: 0
tags:
- App
---

# Character Changelog

Instead of outputting MutatorImpact from CharacterHandle dispatches, output a mutation trait object (something which implements a new Mutation trait). The impl will provide an api for accessing the formatted name of the change (which becomes the commit desc), and a function to mutate a persistent state. If any two adjacent mutations can be merged, they are merged by the changelog (i.e. two hit point mutations with the same "cause").
Mutations can be read from a commit desc (short desc is user-facing, long desc is kdl data).
Mutations cannot be automatically reverted, but can be shown in app UI so users can manually make changes which undo a mutation.
