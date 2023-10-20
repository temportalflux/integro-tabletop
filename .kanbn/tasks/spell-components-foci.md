---
created: 2023-09-20T13:50:56.139Z
updated: 2023-09-20T13:51:07.162Z
assigned: ""
progress: 0
tags: []
---

# Spell Components & Foci

- how do Foci fit into this?
- items can have the `SpellComponent` tag
- spells which have spell components specify the name, an optional gold amount, and if there is a gold amount, optionally consume it
- spells with material components are displayed in the spell ui
- a spell is only castable if you have the components for it. if there is no gold amount, it can be covered by a component pouch or spell casting focus. if there is a gold amount, you must have a matching `SpellComponent` item with the same name and a gold amount >= the required amount.
- if a spell consumes one of these components w/ gold amount, the equivalent gold amount of items is removed from the inventory on cast.
- show in spell overview? (used in spells panel and in spell containers)
```
component "Material" {
	item "Feather"
	item "Pearl" {
		worth 100 (Currency)"Gold" consume=true
	}
}
```
