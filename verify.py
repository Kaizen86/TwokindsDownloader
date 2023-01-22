#!/usr/bin/python3
from PIL import Image
from os import chdir, listdir

chdir("download")

for dir in [d for d in listdir() if "." not in d]:
	for file in listdir(dir):
		image = Image.open(f"{dir}/{file}")
		try:
			image.verify()
		except:
			print(f"BAD FILE {dir}/{file}")
