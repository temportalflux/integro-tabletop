---
created: 2023-09-29T22:25:16.360Z
updated: 2024-07-06T13:17:01.216Z
assigned: ""
progress: 0
tags:
  - Capability
---

# Add "restore_uses" mutator

Mutator which restores some number of uses to a resource when a rest of a particular type is taken. (e.g. Sorcerer's "Sorcerous Restoration")
```
mutator "restore_uses" reset_on="Short" {
	amount 4
	resource "Sorcerer/level02/Font of Magic"
}
```
