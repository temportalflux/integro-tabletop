---
created: 2024-07-11T13:30:06.692Z
updated: 2024-07-11T13:31:37.529Z
assigned: ""
progress: 0
tags:
  - Capability
---

# Mutator to grant uses on rest

e.g. Sorcerer's "Sorcerous Restoration"
`At 20th level, you regain 4 expended sorcery points whenever you finish a short rest.`
```
mutator-todo-qol "restore_uses" reset_on="Short" {
	amount 4
	resource "/Sorcerer/level02/Font of Magic"
}
```
