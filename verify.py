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
args = parser.parse_args()

os.chdir(args.directory)

for chapter in [d for d in os.listdir() if "." not in d]:
  for file in os.listdir(chapter):
    path = "/".join((chapter,file))

    try:
      image = Image.open(path)
      image.verify()
    except OSError:
      print(f"TRUNCATED FILE {path}")
    except:
      print(f"BAD FILE {path}")
    else:
      if args.verbose:
        print(f"OK FILE {path}")
