# Splits a compedium (1 file with many kdl node entries) into multiple files,
# where each relevant node is annotated with a `//!<name>` prefix.
from pathlib import Path
import sys
import os

# Usage:
# python ./scripts/decompend.py ./modulename/dnd5e/items/magic.kdl ./modulename/dnd5e/items/magic/

compendium_file = Path(sys.argv[1])
dst_dir = Path(sys.argv[2])

files = {}
last_name = None
latest_segment = None
left_over = ""
with open(compendium_file, 'r', encoding="utf-8") as compendium:
	for line in compendium:
		if line.startswith('//!'):
			if last_name is not None:
				files[last_name] = latest_segment
			last_name = line.removeprefix('//!').strip()
			latest_segment = ""
		elif latest_segment is not None:
			latest_segment += line
		else:
			left_over += line
if last_name is not None and latest_segment is not None:
	files[last_name] = latest_segment

for name,content in files.items():
	dst_file = dst_dir.joinpath(name + ".kdl")
	os.makedirs(os.path.dirname(dst_file), exist_ok=True)
	dst_file.write_text(content, encoding='utf-8')
compendium_file.write_text(left_over, encoding='utf-8')