---
created: 2024-02-25T19:37:58.579Z
updated: 2024-03-01T16:03:47.989Z
assigned: ""
progress: 0
tags:
  - MVP
started: 2024-03-01T16:03:47.990Z
---

# Generator module components

Modules can define generators, which create new variant entries based on a provided kdl or already parsed components.
All generators are processed AFTER other components (feats, classes, items, etc). Generator processing order is first "kdl" then "item", where generators created by generators are inserted in priority order according to their type (kdl generators which create kdl generators are put tehir results at the end of the kdl queue, and kdl generators which create item generators are put their results at the end of the item generator queue).
`generator "kdl"` applies each `variant` (which contains key-value pairs) to all of the stringify versions of each block inside `base`, replacing keys with values, and then reinterpretting that replaced string as kdl. This can result in other generators being created.
`generator "item"` takes one or more "filter" blocks and looks at all items that have been generated which match that criteria. Their `variant` blocks define operations to apply to all items which pass the filter, to create new items. Names are always changed for example.
