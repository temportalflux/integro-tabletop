# Backlog
-----

## Descriptive Modals
Modals for all elements with tooltips & more.

### Ability Scores
- bonuses & sources
- description
- skills
- other uses (dex for initiative and AC)

### Saving Throw Proficiencies (& Modifiers)
- modal encompases them all
- description of saving throws in general
- each saving throw value, & if its proficient (+sources)
- modifiers (+sources)

### Each Skill
- each has its own modal
- description of skill
- proficiency source
- modifier + source (e.g. disadv on stealth with heavy armor)

### Prof Bons
- description of use

### Armor Class
- description
- formula list (and evaluations) + sources
	e.g. `11 + Dex (+2) + Con (+1) = 14 (Barbarian > Unarmored Defense)`

### Initiative Bonus
- description
- bonuses & sources

### Speeds & Senses
- values & sources
- description of speeds & senses

### Hit Points
- Max HP sumation + sources
- Larger HP modification interface

### Defenses
- Long-form description, split into sections (resistant, immune, vulnerable) with values and their sources

### Conditions
- Add/Remove conditions

### Other
- Each Item
- Each Attack
- Each Feature

## Attacks per Action
Track Attacks per Action & display in attacks pane

## Actions in Combat
- section which lists all the actions that can be used in combat
- Attack, Cast a Spell, Dash, Disengage, Dodge, Grapple, Help, Hide, Improvise, Ready, Search, Shove, Use an Object
- each of these can be clicked on to open a modal with their description

## Classes
- Add hard-coded class content for each class (Barbarian, Bard, Cleric, Druid, Fighter, Monk, Paladin, Ranger, Rogue, Sorcerer, Warlock, Wizard)
- Add 1 subclass for each class in content

## Item Containers
- Only items in the characters main inventory can be equipped
- Weight Capacity (optional)
- Entries (list of items)

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

## Save Data
- Save persistent data to session storage while character is open
- eventually have this all query against github repos where classes, subclasses, backgrounds, lineages, upbringings, items, & spells all have a `repo`+`path` tuple which describes from where to fetch data updates from
- modules can be added to the user's profile to be loaded when the application starts up (by querying the repos)

## Builder/Editor
- select which modules are allowed
- selecting lineage, upbringing, background, class
- levels can be managed here
- Special Level Up button / interface for adding 1 level; includes feature selections and subclass, and multiclassing eventually

## Mailbox
- Pull requests are used to send items between characters
- Characters in other repos can be added as friends (so mail can be sent)
