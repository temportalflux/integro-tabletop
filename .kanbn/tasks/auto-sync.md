---
created: 2023-09-20T13:44:47.322Z
updated: 2023-10-14T12:46:03.859Z
assigned: ""
progress: 0
tags:
  - App
  - MVP
started: 2023-10-07T13:19:55.986Z
completed: 2023-10-14T12:46:03.859Z
---

# Auto-Sync

Check for module updates on app resume/open (post login), automatically sync before app is shown.

Authentication success is the earliest point after "new session" that syncing can be performed. When auth success is detected, a flag should be added to the session indicating that a sync is required.
When the modules or character list pages are opened (first mount), and the sync flag is set, then we should start a autosync.

Document's `visibilitychange` event can detect when the app resumes after being hidden. When the user resumes session with a character sheet open, we should ping the latest version in storage. If the latest version is not the local version, we must sync to latest for the character's module/repo.

When autosyncing, the full page becomes a loading spinner which communicates how many modules are being synced and how many files in each.
