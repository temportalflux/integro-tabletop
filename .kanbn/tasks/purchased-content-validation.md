---
created: 2024-07-19T17:50:13.084Z
updated: 2024-07-20T21:34:10.370Z
assigned: ""
progress: 0
tags:
  - App
---

# Purchased Content Validation

Some systems are primarily sold on DriveThruRPG (e.g. Dungeon World). DTRPG has an API which can be used to validate purchases. Therefore the following can be achieved.

- Modules can be linked to no more than 1 DTRPG product
- Users can link their account (user data) with a DTRPG app key (can be generated in DTRPG account settings)
- A mapping of DTRPG product to one or more modules can be stored somewhere, and users can be granted access to modules if theyve purchased the product



How are user accounts aware of private repos that arent in the integro-tabletop github organization? Ideally product owners can have their own repos for their content (or can be transcribed by third-parties). There would need to be a mechanism to granting access to users via product key.

- Private Module Repos give integro permissions to read the repo and invite others
- Integro accounts have a DTRPG integration field, which allow users to input their DTRPG app/api key
- Integro has a page/dialog that allows users to input a product code (or perhaps scan their DTRPG account)
- Integro can scan private module repos for any which provide a product key (options: module metadata file, repo description, repo tag)
- When integro detects that a user had redeemed a product that it has an equivalent module for, the user is added to the list of github repo readers for that module
- Github users must accept the invite to the repository, so the app will need to display this to users

Resources

- [DriveThruRPG API docs](https://github.com/jramboz/DTRPG_API)
- [.Net API impl](https://github.com/JamesSkemp/DriveThruRpg)
- [python API impl](https://github.com/glujan/drpg)

- [github add contributor](https://docs.github.com/en/rest/collaborators/collaborators?apiVersion=2022-11-28#add-a-repository-collaborator)
- [github accept contributor invite](https://docs.github.com/en/rest/collaborators/invitations?apiVersion=2022-11-28)
