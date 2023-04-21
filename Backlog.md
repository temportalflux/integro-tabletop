# Backlog
-----

## IndexedDB
- force refresh button to forcibly reload one or more modules by wiping those entries from the database and refetching them from source

## UI Backlog
- Limited Use numerical modifier (for max uses > 5)
- UI components for rendering features, conditions, mutators, etc which is used for all presentations (mutators / features in origin & item browsers, condition cards, feature/action modals, etc)

## Unify mutators, criteria, and features
- Convert `feature` block to a `mutator "feature"` block, features are granted using mutators. FromKDL structs which accept both mutators and features now only accept mutators.
- Convert `criteria` into a `mutator "only_if"` (or similar name) which accepts a single criteria and any number of submutators. Submutations are only applied if the criteria passes. This replaces the usage of `criteria` in features and conditions.

## Conditions
- degrees (like exhaustion): these are stages of the same condition, which add more mutators the higher the degree.

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
