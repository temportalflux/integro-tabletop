from pathlib import Path
import sys
import os

# Usage:
# python ./scripts/renameCompendiumGroups.py ./modulename/dnd5e/items/magic.kdl

# Replaces the values at every `//!<value>` to be in camelcase w/o spaces

compendium_file = Path(sys.argv[1])
compendium_txt = ""
with open(compendium_file, 'r', encoding="utf-8") as compendium:
	for line in compendium:
		if line.startswith('//!'):
			groupName = line.removeprefix('//!').strip()
			words = groupName.split(" ")
			words = [''.join(filter(str.isalpha, word)) for word in words]
			compendium_txt += "//!" + words[0].lower() + ''.join(c.capitalize() for c in words[1:]) + '\n'
		else:
			compendium_txt += line
compendium_file.write_text(compendium_txt, encoding='utf-8')
