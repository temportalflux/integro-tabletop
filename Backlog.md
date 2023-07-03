# Backlog
-----

## Cleanup
- proficiency level and its html should be separate functions
- kdl NodeReader to combine KdlNode and NodeContext (mainly for consume_idx and next_node)

## DnD5e Features
- Starting Equipment in UI
- Showing weapon attacks with the additional bonuses in atk rolls and damage (UI doesn't support this right now)
- Condition degrees (like exhaustion): these are stages of the same condition, which add more mutators the higher the degree.
- Attunement; 3 slots per character, can select attunable items in the character's equipment
- Item Charges
- Sheet inventory search bar functionality
- name generator
- Warlock Invocations & Sorcerer Metamagic selections

### Rest
- Short and Long rest buttons with functionality
- Things that get updated on rest
	- anything with `LimitedUses` where `reset_on` is specified (features/actions, always-prepared spellcasting)
	- hit points
	- spell slots
- Rest modal should tell the user what all is changing for a given rest (things affected get registered on the character)
- hit points UI should track hit dice (for usage during short rest or when features specify like Wither and Bloom)

### Customizations
- Allow users to create new entries
	- Features & Actions
	- Conditions
	- Saving Throw Modifiers
	- Skills
	- Other Proficiencies
	- Defenses
	- Feats
- Homebrew; Users can duplicate any existing entry to their homebrew content, which can then be editted

### Character Modules
- Each character has a list of modules that are used
- Populated with defaults specified in the user's settings when a new character is created
- Can add or remove modules in the editor for a character

### Spell Components
- how do Foci fit into this?
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

### Spell Containers
- Preparation Source (can spells be prepared from here, i.e. spellbook for wizard)
- Max Spell Count (optional); max number of spells this container can hold (e.g. spell gems & scrolls only hold one)
- Max Level Per Spell (optional); max level any spell in this container can be (e.g. spell gems have a cap on the tier of spell)
- Max Total Level (optional): max value of the sum of all spell levels in this container (e.g. ring of spell storing has a general cap on all stored spells)
- Entries (list of spells)
- Some spell containers can only be transcribed from, never prepare or cast from

### Views
- Desktop
	- Display
		- Description
- Mobile
	- Display
		- Header
		- Speeds, Senses, & Other Proficiencies
		- Combat
		- Actions & Features
		- Spells
		- Inventory
		- Description
	- Editor; TBD

## App Features

### Modules page
- view all locally installed modules
- (action) force refresh to delete installation and reinstall (per module or all)
- (action) check for updates; query for any new revisions/commits
- (action) clone module from its source to a user's own homebrew

## Documentation
- README w/ feature comparison against other character sheet apps
- mdbook on content syntax, mutators, etc (for creating content by hand)
- walkthrough embedded in site showing users how character sheets work
- mdbook on the tech stack & functionality of the app

## Future Features

### Mailbox
- Pull requests are used to send items between characters
- Characters in other repos can be added as friends (so mail can be sent)

### User-Written Modules
- users can create new modules, saved to github (or other backend servicer)
- modules are querried from backend(s) on app load, and opt-in able for any given character
- modules have permissions (based on backend) and user access can be added by the module owner via the app
- App has interface support for editing modules (adding/removing content, updating content with versioning)

### Standalone app?
- maybe run wasm in a winit window via https://docs.wasmtime.dev/
