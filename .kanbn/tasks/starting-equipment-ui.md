---
created: 2023-09-20T13:29:02.294Z
updated: 2023-09-23T16:56:37.492Z
assigned: ""
progress: 0
tags:
  - MVP
started: 2023-09-22T00:00:00.000Z
---

# Starting Equipment UI

Characters without equipment are shown a prompt which allows them to pick their starting equipment based on mutators applied via their background.

Group:
- description states how many to pick
- each option has a checkbox (similar to spellslots)
- if the number of checked options meets or exceeds the number to pick, all unchecked options are disabled
- if the number to pick is 1, checking any box unchecks all others
- the contents of each option are disabled if the option is not checked
- contents can probably be displayed in a row-flex-box with the checkbox and title above its contents

SelectItem
- dropdown with items which match the filter criteria

IndirectItem & SelectItem both display the resulting item as a name + quantity, with a button opening that item in a child details panel
