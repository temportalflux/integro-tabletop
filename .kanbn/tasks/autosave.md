---
created: 2023-09-20T13:43:40.933Z
updated: 2023-09-20T13:44:25.090Z
assigned: ""
progress: 0
tags:
  - App
  - Storage
  - MVP
---

# Autosave

- Leaving editor always saves
- All changes generate a changelog message
- Adjacent changelog messages with the same type will auto-combine (e.g. hit point increment & decrement, excluding bulk changes)
- If there are no additional changes within 60 seconds, auto save
- Display save timer and a manual save button in header of display sheet under the rest & builder buttons
- Button to open changelog and view both committed and uncommitted changes
