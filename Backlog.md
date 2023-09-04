# Backlog
-----

## App Flow Improvements
- Check for module updates on app resume/open (post login), automatically sync before app is shown
- Toast in bottom right for active tasks
- Saving to Storage
	- Leaving editor always saves
	- All changes generate a changelog message
	- Adjacent changelog messages with the same type will auto-combine (e.g. hit point increment & decrement, excluding bulk changes)
	- If there are no additional changes within 60 seconds, auto save
	- Display save timer and a manual save button in header of display sheet under the rest & builder buttons
	- Button to open changelog and view both committed and uncommitted changes
- Modal Replacement
	For contexts which currently use a modal, where the user could drill down to an inner page (e.g. bundle selection, item and spell containers, etc),
	we should not use a modal stack. Modals should be used exclusively for confirmation or form popups. Instead, the character sheet should have the concept of a "context panel". D&DBeyond uses one on the right side. For this implementation, the Paged character sheet (default on mobile, opt in on desktop) this is easy. The paged character sheet will just have a full-screen drill down panel, which displays a breadcrumb path on the top with a close button. When you drill down again, the breadcrumb length increases and the button becomes a back button (you have to fully back out to close the panel). For the Joined sheet (default on desktop), the context panel will slide up from the bottom. If the user clicks outside of the panel, it closes. Its basically a drawer. Maybe the user can also reopen it if it retains its path/state?
	Anytime we open a modal on the sheet should use the context panel instead.

## Tech Debt
- Additional Object Cache should save its bundles to the character kdl (and read them) so offline character sheets can be supported. Ideally we'd track what bundles are actually being used by the charcter so we don't reserialize bundles that aren't actively in use.
- Right now, evaluator's output type must be the same as the type read from kdl. Instead, an evaluator should have a parsed type (bool, int, float, string) and an output type (same as parsed or impls From/FromStr of the underlying type). This would support something like `Value<Rest>` instead of needing to use `Rest::from_str(Value<String>::evaluate())`. The output type can be lazily provided at evaluation time, instead of being a compile-time restriction. Heck, at that rate we dont even need a compile-time parsed type because it can be stored as a KdlValue (enum wrapper around all primitive types), which is then parsed at evaluation time. This goes back to the initial implementation debate of is it better to ensure the input data is accurate at parse time or eval time, and given the pain it has been to create non-primitive typed evals, I think i'd be better to find a middle ground than going to one extreme or the other. In that world, the parsed and output types don't matter at compile time, but when the evaluator parses kdl and is evaluated, both of those functions should take the expected type so that the kdl can be checked at parse time, stored as a general primitive, and then parsed again at eval time.

## DnD5e Features

- Wizards can add spells to spellbook
  req; spell containers
- Wizard's can prepare spells from spellbook
  req; Wizards can add spells to spellbook

- Apply Dodge condition when the action is taken (defined in `defaults.kdl`)
- Expertise - selection filter to select skills that the character already has proficiency in
- Conditions imply other Conditions
- Ranger Favored Enemy and Natural Explorer
- Condition supression (`Mindless Rage`)
- Granting speed based on another speed (`Second-Story Work`, `Dragon Wings`)
- Bonus to damage from spells based on a criteria (`Empowered Evocation`)
- Starting Equipment in UI
- Showing weapon attacks with the additional bonuses in atk rolls and damage (UI doesn't support this right now)
- Condition degrees (like exhaustion): these are stages of the same condition, which add more mutators the higher the degree.
- Attunement; 3 slots per character, can select attunable items in the character's equipment
- Item Charges
- Sheet inventory search bar functionality
- name generator
- Selected Eldritch Invocations should show the bundle descriptions in the panel under the feature, like a feature with a parent

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
- show in spell overview? (used in spells panel and in spell containers)
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

## App Features

- Character History / Changelog
	Instead of outputting MutatorImpact from CharacterHandle dispatches, output a mutation trait object (something which implements a new Mutation trait). The impl will provide an api for accessing the formatted name of the change (which becomes the commit desc), and a function to mutate a persistent state. If any two adjacent mutations can be merged, they are merged by the changelog (i.e. two hit point mutations with the same "cause").
	Mutations can be read from a commit desc (short desc is user-facing, long desc is kdl data).
	Mutations cannot be automatically reverted, but can be shown in app UI so users can manually make changes which undo a mutation.

### Modules page
- Can open a module modal to delete or perform other actions
- Deleting a module requires that it is not used by any locally installed characters
- Modules can be in the database without being installed. Users can select which modules they install out of all those they have access to.
- (action) check for updates; query for any new revisions/commits
- (action) clone module from its source to a user's own homebrew

## Documentation
- README w/ feature comparison against other character sheet apps
- mdbook on content syntax, mutators, etc (for creating content by hand)
- walkthrough embedded in site showing users how character sheets work
- mdbook on the tech stack & functionality of the app

## Future Features

- Hit Dice Mutator: instead of each class hard-coding its hit die, classes have a mutator which add hit die of a type based on the level of that class. This will enable other content or feats to also add hit dice if desired. It also allows for features to consume hit dice as a usage/resource.

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
- Perhaps with Tauri?
	https://tauri.app/v1/guides/getting-started/setup/integrate
	https://dev.to/stevepryde/create-a-desktop-app-in-rust-using-tauri-and-yew-2bhe
