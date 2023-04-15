# Backlog
-----

## IndexedDB
- need to store modules locally (so they dont have to be fetched every time the app is opened)
- localstorage is not big enough
- use indexeddb via https://github.com/devashishdxt/rexie or https://github.com/devashishdxt/idb

## Feature UI
- collapse features with "category" property into one section,
  where the name of a feature which is also a category for other features (e.g. `Ki`) is the root element.
- smaller text
- use short description; if no short text, collapse feature to only name & source
- open modal on click
	- source
  - long description
  - mutators & criteria

## Action UI
- collapse features with "category" property into one section,
  where the name of a feature which is also a category for other features (e.g. `Ki`) is the root element.
- use short description; if no short text, collapse feature to only name & source
- open modal
	- source
  - long description
	- attack
	- applied conditions
	- limited uses

- Limited Use numerical modifier (for max uses > 5)
- Condition UI lists mutators

## Attunement UI
- 3 slots to which attunable items can be "equipped"
- must be in the top-level inventory (equipment)

## Spellcasting
- Fill out the spells panel
- a spellcasting feature populates some top-level derived optional struct for spellcasting
- persistent data stores what spells are prepared (vs the feature or containers which store what is known/learned)

## Spell Containers
- Preparation Source (can spells be prepared from here, i.e. spellbook for wizard)
- Max Spell Count (optional); max number of spells this container can hold (e.g. spell gems & scrolls only hold one)
- Max Level Per Spell (optional); max level any spell in this container can be (e.g. spell gems have a cap on the tier of spell)
- Max Total Level (optional): max value of the sum of all spell levels in this container (e.g. ring of spell storing has a general cap on all stored spells)
- Entries (list of spells)

## Description
- Gender/Pronouns
- Size
- Height + Weight
- Age
- Appearance
- Personality Traits, Ideals, Bonds, Flaws

## Customizations
Allow users to create new elements
- Actions
- Conditions
- Saving Throw Modifiers
- Skills
- Other Proficiencies
- Defenses

## Serialization
- `trait AsKDL` & `trait FromKDL` to handling kdl text <=> structs

## Save Data
- Save persistent data to session storage while character is open
- eventually have this all query against github repos where classes, subclasses, backgrounds, lineages, upbringings, items, & spells all have a `repo`+`path` tuple which describes from where to fetch data updates from
- modules can be added to the user's profile to be loaded when the application starts up (by querying the repos)

## Mailbox
- Pull requests are used to send items between characters
- Characters in other repos can be added as friends (so mail can be sent)

## Write Modules
written in kdl, hosted in github
- D&D Basic Rules
- Elf and an Orc had a little baby (v2?)
- Other Official D&D Content

## User-Written Modules
- users can create new modules, saved to github (or other backend servicer)
- modules are querried from backend(s) on app load, and opt-in able for any given character
- modules have permissions (based on backend) and user access can be added by the module owner via the app
- App has interface support for editing modules (adding/removing content, updating content with versioning)

## Standalone app?
- maybe run wasm in a winit window via https://docs.wasmtime.dev/
