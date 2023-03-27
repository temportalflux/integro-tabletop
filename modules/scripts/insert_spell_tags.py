import os
from pathlib import Path
import re
import sys
from typing import Dict, List

# Usage: python <script> <path to tag list file> <path to spells directory>
# Tag List Format:
# ```
# # <class name>
# 	<spell name>
# 	<spell name>
# # <class name>
# 	<spell name>
# 	<spell name>
# 	<spell name>
# ```

tag_list = Path(sys.argv[1])
spell_dir = Path(sys.argv[2])

tags_by_spell_name: Dict[str, List[str]] = {}
current_class_tag = None
with open(tag_list, 'r') as tag_list:
	for line in tag_list:
		if line.startswith('# '):
			current_class_tag = line.removeprefix('# ').strip()
		elif line.startswith('\t'):
			spell_name = line.strip()
			if current_class_tag is not None:
				if not tags_by_spell_name.__contains__(spell_name):
					tags_by_spell_name[spell_name] = list()
				tags_by_spell_name[spell_name].append(current_class_tag)

#for spell_name, tags in tags_by_spell_name.items():
#	print(spell_name)
#	for tag in tags:
#		print(f"\t{tag}")

# quit(0)

re_spell_name = re.compile('spell name="(.*)"')
for root, dirs, files in os.walk(spell_dir):
	for file_name in files:
		path = Path(os.path.join(root, file_name))
		content = path.read_text(encoding = "utf-8")
		re_match = re_spell_name.match(content)
		if re_match is not None:
			spell_name = re_match.group(1)
			if tags_by_spell_name.__contains__(spell_name):
				#print(f"{spell_name} => {tags_by_spell_name[spell_name]}")
				tag_lines = [f'\ttag "{tag}"' for tag in tags_by_spell_name[spell_name]]
				tag_lines = '\n'.join(tag_lines)
				# the tags are inserted right before spell rank
				new_content = content.replace('\n\trank', f"\n{tag_lines}\n\trank")
				path.write_text(new_content, encoding="utf-8")
