---
created: 2024-07-19T17:50:13.084Z
updated: 2024-07-19T17:50:13.082Z
assigned: ""
progress: 0
tags: []
---

# Purchased Content Validation

Some systems are primarily sold on DriveThruRPG (e.g. Dungeon World). DTRPG has an API which can be used to validate purchases. Therefore the following can be achieved.

- Modules can be linked to no more than 1 DTRPG product
- Users can link their account (user data) with a DTRPG app key (can be generated in DTRPG account settings)
- A mapping of DTRPG product to one or more modules can be stored somewhere, and users can be granted access to modules if theyve purchased the product

Yet unknowns

- How are user accounts aware of private repos that arent in the integro-tabletop github organization? Ideally product owners can have their own repos for their content (or can be transcribed by third-parties). There would need to be a mechanism to granting access to users via product key.

Resources

- [DriveThruRPG API docs](https://github.com/jramboz/DTRPG_API)
- [.Net API impl](https://github.com/JamesSkemp/DriveThruRpg)
- [python API impl](https://github.com/glujan/drpg)

