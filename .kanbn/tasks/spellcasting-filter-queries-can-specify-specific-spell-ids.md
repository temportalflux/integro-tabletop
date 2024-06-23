---
created: 2024-03-23T16:29:02.851Z
updated: 2024-06-21T15:58:29.768Z
assigned: ""
progress: 0
tags:
  - MVP
completed: 2024-06-21T15:58:29.768Z
---

# spellcasting::Filter queries can specify specific spell ids

```
mutator-todo "modify" (Spell)"Damage" {
	damage (Evaluator)"get_ability_modifier" (Ability)"Charisma"
	// TODO: spellcasting::Filter queries can specify specific spell ids
	query {
		spell "spells/eldritchBlast.kdl"
	}
}
```
