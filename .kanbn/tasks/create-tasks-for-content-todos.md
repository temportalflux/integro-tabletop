---
created: 2023-09-29T23:17:28.444Z
updated: 2023-09-29T23:18:01.766Z
assigned: ""
progress: 0
tags: []
---

# Create tasks for content todos

The major ones are:
```
Wizard "Spell Mastery"
- Mutator which has a selection for a 1st and 2nd level Wizard spell from their spellbook, and grants at-will casting for those spells at their minimum rank.

Beserker Barbarian "Frenzy"
- User-Triggerable button which is available if the character has the Raging condition
- Apply a level of Exhaustion when the Raging condition ends
- All melee weapon attacks can be performed using a bonus action (in addition to existing action kind)

Beserker Barbarian "Mindless Rage"
- While raging is active, supress the charmed and frightened conditions

Spells can grant healing
Speed can be overriden to 0 via conditions

Life Cleric "Disciple of Life"
- Grant an extra (evaluated) amount of hp per spell that grants healing (2 + the spell's rank)

Open Hand Monk "Tranquility"
- "Sanctuary" spell has a condition that it grants
- Grant the same condition when a long rest ends (which goes away when a long rest is taken)

Devotion Paladin "Purity of Spirit"
- "Protection from Evil and Good" spell has a condition that it grants
- feature ensures that the character always has that condition (cannot be removed)

Bundle's can have a "minimum level" (class or subclass) requirement
- Eldritch Invocations: ascendantStep, bewitchingWhispers, chainsOfCarceri, dreadfulWord, lifedrinker, masterOfMyriadForms, minionsOfChaos, mireTheMind, oneWithShadows, otherworldlyLeap, sculptorOfFlesh, signOfIllOmen, thirstingBlade, visionsOfDistantRealms, whispersOfTheGrave, witchSight

Bundle requirement evaluator for having a cantrip/spell
- Eldritch Invocations: agonizingBlast, eldritchSpear, repellingBlast

Bundle requirement "has_selection" (takes path and specific value required)
- Eldritch Invocations: bookOfAncientSecrets, chainsOfCarceri, lifedrinker, thirstingBlade, voiceOfTheChainMaster

Spellcasting which uses a slot for a specific spell, and has limited uses
- Eldritch Invocations: bewitchingWhispers, dreadfulWord, minionsOfChaos, mireTheMind, sculptorOfFlesh, signOfIllOmen, thiefOfFiveFates

warlock/invocations/
	agonizingBlast
	- Bonus damage for specific spells
	bookOfAncientSecrets
	- provides unusual spellcasting feature
	chainsOfCarceri
	- spellcasting against specific creature types w/ limited uses
	eldritchSpear
	- Bonus range for specific spells
	lifedrinker
	- some way to identify what the Pact Weapon is in the inventory (tag on weapon? selection in Pact of the Blade feature?)
	- bonus damage for specifically the Pact Weapon
	mireTheMind
	- custom button which grants invisibility when character is in dim light or darkness
	repellingBlast
	- grant extra notes to a specific spell
	thirstingBlade
	- grant extra attack with Pact Weapon

Equipment items can have criteria for being equipped (e.g. checking an ability score)
Belt of * Giant Strength: mutator which applies a minimum bound for an ability score

Hobgoblin "Reserves of Strength"
- bonus damage if weapon is strength-based melee
```
