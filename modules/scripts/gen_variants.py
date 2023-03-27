import glob
import os
from pathlib import Path
import shutil
import sys
import json

if __name__ == '__main__':
	root = Path(os.getcwd()).joinpath(sys.argv[1])
	template_name = sys.argv[2]
	template_dir = root.joinpath(template_name)
	template_path = template_dir.joinpath("_base.kdl_template")

	with open(template_dir.joinpath("_variants.json"), 'r') as file:
		variants = json.load(file)

	for f in glob.glob(str(template_dir.joinpath('*.kdl'))):
		try:
			os.remove(f)
		except OSError as e:
			pass

	with open(template_path, 'r') as file:
		template_txt = file.read()

	for name, replacements in variants.items():
		variant_path = template_dir.joinpath(name + ".kdl")
		variant_txt = template_txt
		for key, value in replacements.items():
			variant_txt = variant_txt.replace(f"{{{key}}}", value)
		with open(variant_path, 'w') as file:
			file.write(variant_txt)
	
	if os.path.exists(template_dir.joinpath("__pycache__")):
		shutil.rmtree(template_dir.joinpath("__pycache__"))
			
