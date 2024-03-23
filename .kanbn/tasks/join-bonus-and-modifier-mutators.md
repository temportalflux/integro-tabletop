---
created: 2024-03-23T14:53:08.129Z
updated: 2024-03-23T17:04:44.673Z
assigned: ""
progress: 0
tags:
  - MVP
started: 2024-03-23T14:54:25.446Z
completed: 2024-03-23T17:04:44.673Z
---

# Join bonus and modifier mutators

Really they are one mutator, something which changes stats using a number or roll modifier. 
```
mutator "modify" (Ability)"All/Specific" "[Ability]"
mutator "modify" (SavingThrow)"All/Specific" "[Ability]" context?=""
mutator "modify" (Initiative)"[Disa/A]dvantage"
mutator "modify" (Skill)"Specific" "[Skill]"
mutator "modify" (Attack)"Roll" {
    bonus #
    modifier "[Disa/A]dvantage"
    ability "[Ability]"
    query ...
}
mutator "modify" (Attack)"Damage" {
    damage ...
    query ...
}
mutator "modify" "ArmorClass" #
mutator "modify" (Spell)"Damage" {
    damage ...
    query ...
}
```
