#!/usr/bin/python3
import argparse
import os
from PIL import Image

parser = argparse.ArgumentParser(
  prog = os.path.basename(__file__),
  description = "Checks all downloaded images to make sure there aren't any corrupted files"
)

parser.add_argument("-d", "--directory",
  dest="directory", default="download", help="Download directory to scan")
parser.add_argument("-v", "--verbose",
  dest="verbose", action="store_true", help="Include verified files in output")
parser.add_argument("-c", "--cleanup",
  dest="cleanup", action="store_true", help="Delete any corrupted images automatically")
args = parser.parse_args()

os.chdir(args.directory)
error_count = 0
print("Scanning - this may take a while")

for chapter in [d for d in os.listdir() if "." not in d]:
  for file in os.listdir(chapter):
    path = "/".join((chapter,file))

    # Both tests yield OSErrors, so I'll have to do some try/except gymnastics here.
    bad_file = False
    try:
      # Opening the file can detect the most obvious errors with file headers
      image = Image.open(path)
    except OSError:
      print(f"TRUNCATED FILE {path}")
      bad_file = True

    if not bad_file:
      try:
        # This is a more thorough check, forcing Pillow to read the entire file
        image.transpose(Image.FLIP_LEFT_RIGHT)
      except:
        print(f"BAD FILE {path}")
        bad_file = True
      else:
        if args.verbose:
          print(f"OK FILE {path}")

    if bad_file:
      error_count += 1
      if args.cleanup:
        os.remove(path)

if error_count > 0:
  suffix = (" and cleaned up." if args.cleanup else
     ".\nRecommend using -c to clean them up and then re-running the scraper.")
else:
  suffix = ". :)"
print(f"{error_count} corrupt image{'s'*(error_count!=1)} found" + suffix)
