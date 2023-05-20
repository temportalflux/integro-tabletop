# Backlog
-----

## IndexedDB
- force refresh button to forcibly reload one or more modules by wiping those entries from the database and refetching them from source

## UI Backlog
- PickN mutator description (name, body, selector metadata)
- Limited Use can use the resource of another feature/action by its path, and has an associated cost when doing so
- UI components for rendering features, conditions, mutators, etc which is used for all presentations (mutators / features in origin & item browsers, condition cards, feature/action modals, etc)

## Unify mutators, criteria, and features
- Convert `feature` block to a `mutator-todo "feature"` block, features are granted using mutators. FromKDL structs which accept both mutators and features now only accept mutators.
- Convert `criteria` into a `mutator "only_if"` (or similar name) which accepts a single criteria and any number of submutators. Submutations are only applied if the criteria passes. This replaces the usage of `criteria` in features and conditions.

## Conditions
- degrees (like exhaustion): these are stages of the same condition, which add more mutators the higher the degree.

## Attunement UI
- 3 slots to which attunable items can be "equipped"
- must be in the top-level inventory (equipment)

## Rest
- Short and Long rest buttons with functionality
- make sure to clear consumed spell slots based on rest and caster data

## Spellcasting
- Spell panel rows for selected spells
	- cast vs use buttons
- non-stub search functionality in main panel
- Spell management
	- search available spells to be selected
- ritual & focus functionality
- limited uses for add_prepared (self-defined, using another feature, or charges)
- add_prepared spells cost (at-will vs requires_slot vs limited_use)
- Spell Components
	- items can have the `SpellComponent` tag
	- spells which have spell components specify the name, an optional gold amount, and if there is a gold amount, optionally consume it
	- spells with material components are displayed in the spell ui
	- a spell is only castable if you have the components for it. if there is no gold amount, it can be covered by a component pouch or spell casting focus. if there is a gold amount, you must have a matching `SpellComponent` item with the same name and a gold amount >= the required amount.
	- if a spell consumes one of these components w/ gold amount, the equivalent gold amount of items is removed from the inventory on cast.
	```
	component "Material" {
		item "Feather"
		item "Pearl" {
			worth 100 (Currency)"Gold" consume=true
		}
	}
	```

## Spell Containers
- Preparation Source (can spells be prepared from here, i.e. spellbook for wizard)
- Max Spell Count (optional); max number of spells this container can hold (e.g. spell gems & scrolls only hold one)
- Max Level Per Spell (optional); max level any spell in this container can be (e.g. spell gems have a cap on the tier of spell)
- Max Total Level (optional): max value of the sum of all spell levels in this container (e.g. ring of spell storing has a general cap on all stored spells)
- Entries (list of spells)
- Some spell containers can only be transcribed from, never prepare or cast from

## Inventory
- non-stub search functionality

## Performance
- When a character is loaded, it shouldnt require any extra data from modules, it should be able to be fully self contained
	- known problem areas: features which grant innate spellcasting
- load spells and items in the background after the character is loaded
- dont load feature groups (class, race, background, lineage, etc) unless the character needs access to editor. Once a character is created, the user doesn't usually need access to any of the other components that arent inlined into their character data.

## Customizations
Allow users to create new elements
- Actions
- Conditions
- Saving Throw Modifiers
- Skills
- Other Proficiencies
- Defenses
- Feats

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
