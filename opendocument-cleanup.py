#!/usr/bin/env python3
from argparse import ArgumentParser, RawDescriptionHelpFormatter
from os import utime
from time import time
from pathlib import Path
from shutil import copystat
import re, zipfile

# parse arguments
parser = ArgumentParser(
	description="Used to clean up formatting information in OpenDocument files, because LibreOffice doesn't seem to do cleanup when saving on it's own. Focused on cleaning OpenDocument Text files, but also capable of cleaning other OpenDocument formats on a basic level. The cleaning process removes unused or redundant data in the content.xml (document/page content) by default. Your styles and such will not be modified.\nBy default, cleaned files are saved as copies so that you still have the original in case the script somehow messes something up. (It's a good idea to check the cleaned document to make sure it still looks correct, just in case.)",
	epilog="""
values for disposal:
  none          leave untouched and create a copy with '-cleanup' at the end of the name
  trash         move to trash and replace with the cleaned files
  overwrite     overwrite originals (I strongly advise against using this, as this script is not perfect, and may make mistakes)

values for verbosity:
  0     nothing
  1     errors only
  2     errors, error tips, opening/saving
  3     same as 2 but adds data removal stats (default when cleaning multiple files or a directory)
  4     all available info (default when given a single file)

If an unhandled error occurs with the script, check on GitHub to see if it's been fixed recently, or if it hasn't, feel free to post the issue.
""",
	formatter_class=RawDescriptionHelpFormatter
)
parser.add_argument("paths", metavar='path', type=Path, nargs='+', help="path (or paths) to file or directory to clean. If given a directory, ")
parser.add_argument("-d", "--disposal", type=str, choices=['none','trash','overwrite'], default='none', help="what to do with the original file after cleanup (default: none)")
parser.add_argument("-r", "--recursive", action='store_true', help="also search for files in subdirectories when given a directory")
parser.add_argument("-f", "--remove-fonts", action='store_true', help="remove font information (typeface only) from direct formatting (highly suggested if only one font is used in a given file)")
parser.add_argument("-l", "--keep-language", dest="remove_language", action='store_false', help="keep language and country information - (this information is almost never relevant)")
parser.add_argument("-v", "--verbosity", metavar="{0..4}", type=int, choices=range(5), help="how much information will be printed")
args = parser.parse_args()

# import send2trash
if args.disposal == 'trash': from send2trash import send2trash
# default verbosity
if args.verbosity is None: args.verbosity = 3 if len(args.paths) > 1 or args.paths[0].is_dir() else 4

# get files
files = set()
for path in args.paths:
	if path.exists():
		# test if file is a valid document
		def is_document(path : Path) -> bool:
			return path.is_file() and zipfile.is_zipfile(path) and zipfile.Path(path, 'content.xml').exists()
		
		# given directory
		if path.is_dir():
			if args.verbosity >= 3: print("Finding documents...")
			
			def get_documents(dir_path : Path) -> set:
				dir_files = set()
				sub_dir_files = set()
				for path in dir_path.iterdir():
					if is_document(path):
						dir_files.add(path)
						if args.verbosity >= 4: print(f"Found file {path}")
					elif args.recursive and path.is_dir():
						sub_dir_files.update(get_documents(path))
						if args.verbosity >= 4: print(f"Found directory {path.resolve()}")
				if args.verbosity >= 3 and dir_files:
					print(f"Found {len(dir_files)} files in {dir_path.resolve()}")
				return dir_files.union(sub_dir_files)
			
			files.update(get_documents(path))
			
			if args.verbosity:
				if files:
					if args.verbosity >= 2: print(f"Total of {len(files)} files found.")
				elif args.recursive:
					print(f"\33[93mNo documents found in '{path.resolve()}' or any subdirectories.\33[0m")
				else:
					print(f"\33[93mNo documents found in '{path.resolve()}'.\33[0m")
					if args.verbosity >= 2: print("\33[93mUse '-r' or '--recursive' to also search subdirectories.\33[0m")
		# given file
		elif is_document(path): files.add(path)
		# not a document file
		elif args.verbosity:
			print(f"\33[93m'{path.name}' is not a document file\33[0m")
			if args.verbosity >= 2: print("This script can only clean up OpenDocument Text files. (These typically end in .odt or .ott.)")
	
	elif args.verbosity: print_error(f"No such file or directory '{path}'")

# cleanup files
files_cleaned = 0
total_shrink = 0
for path in files:
	# determine destination for saving
	if args.disposal == 'none':
		if path.suffix: save_path = path.with_name(path.name[:-len(path.suffix)] + '-cleaned' + path.name[-len(path.suffix):])
		else: save_path = path + '-cleaned'
	else: save_path = path
	
	# verify and read file
	if args.verbosity >= 4: print(f"\nReading \33[95m{path.name}\33[0m.")
	elif args.verbosity >= 3: print(f"\33[95m{path.name}\33[0m")
	try:
		# check that destination is available
		if args.disposal == 'none' and save_path.exists(): raise Exception(f"'{save_path}' already exists")
		# read content
		content = zipfile.Path(path, 'content.xml').read_text()
	except KeyError: print(f"\33[93m'{path}' is not a document\33[0m, skipping file.")
	except (Exception, FileNotFoundError, OSError) as error: print(f"\33[93m{error}\33[0m, skipping file.")
	# clean document
	else:
		# functions
		def get_element_end(pos : int) -> int:
			tag_end = content.find('>', pos) + 1
			if content[tag_end - 2] == '/': return tag_end
			levels = 1
			while '<' in content[pos:]:
				pos = content.find('<', pos + 1) + 1
				tag_end = content.find('>', pos) + 1
				if content[pos] == '/':
					levels -= 1
					if levels == 0: return tag_end
				elif content[tag_end - 2] != '/': levels += 1
		
		def get_property(pos : int, name : str) -> str:
			begin = content.find(name + '="', pos) + len(name) + 2
			return content[begin : content.find('"', begin)]
		
		def remove_attribute(strings : list, message : str):
			global content
			removed = 0
			while strings[0] in content:
				search_pos = content.find(strings[0])
				end_pos = search_pos + len(strings[0])
				for string in strings[1:]: end_pos = content.index(string, end_pos) + len(string)
				content = content[:search_pos] + content[end_pos:]
				removed += 1
			if args.verbosity >= 3 and removed: print(f"\33[92mRemoved {removed} {message}.\33[0m")
		
		# cleanup
		if args.verbosity >= 4: print("Beginning content cleanup process...\n\nRemoving irrelevant data...")
		remove_attribute([' officeooo:rsid="', '"'], "officeooo:rsid entries")
		remove_attribute([' officeooo:paragraph-rsid="', '"'], "officeooo:paragraph-rsid entries")
		remove_attribute([' loext:opacity="100%"'], "loext:opacity entries")
		
		if args.remove_language:
			if args.verbosity >= 4: print("Searching for languages and countries...")
			remove_attribute([' style:language', '"', '"'], "language entries")
			remove_attribute([' style:country', '"', '"'], "country entries")
		if args.remove_fonts:
			if args.verbosity >= 4: print("Searching for fonts...")
			remove_attribute(['<style:font-face', '/>'], "fonts")
			remove_attribute([' style:font-name="', '"'], "styles")
		
		if args.verbosity >= 4: print("Searching for orphan styles...")
		removed = 0
		pos = 0
		while '<style:style' in content[pos:]:
			# find style
			pos = content.find('<style:style', pos)
			# check if style is used anywhere
			if 'style-name="' + get_property(pos, 'style:name') in content: pos += 1
			else:
				# remove style
				content = content[:pos] + content[get_element_end(pos):]
				removed += 1 
		if args.verbosity >= 3 and removed: print(f"\33[92mRemoved {removed} orphan styles.\33[0m")
		
		if args.verbosity >= 4: print("Searching for orphan list styles...")
		removed = 0
		pos = 0
		while '<text:list-style' in content[pos:]:
			# find style
			pos = content.find('<text:list-style', pos)
			# check if style is used anywhere
			if 'style-name="' + get_property(pos, 'style:name') in content: pos += 1
			else:
				# remove style
				content = content[:pos] + content[get_element_end(pos):]
				removed += 1
		if args.verbosity >= 3 and removed: print(f"\33[92mRemoved {removed} orphan lists styles.\33[0m")
		
		if args.verbosity >= 4: print("Searching for empty styles...")
		removed = 0
		while True:
			# find style
			match = re.search('<style:style style:name="(\w+)" style:family="[a-z]+" style:parent-style-name="(\w+)"(?:/|><style:text-properties/></style:style)>', content)
			if not match: break
			# remove style
			content = content[:match.start()] + content[match.end():]
			content = content.replace(f'style-name="{match[1]}"', f'style-name="{match[2]}"')
			removed += 1
		while True:
			# find style
			match = re.search('<style:style style:name="(\w+)" style:family="text"(?:/|><style:text-properties/></style:style)>', content)
			if not match: break
			# remove style
			content = content[:match.start()] + content[match.end():]
			while f'<text:span text:style-name="{match[1]}">' in content:
				begin = content.find(f'<text:span text:style-name="{match[1]}">')
				end = get_element_end(begin)
				content = content[:begin] + content[content.find('>', begin) + 1 : end - 12] + content[end:]
			removed += 1
		if args.verbosity >= 3 and removed: print(f"\33[92mRemoved {removed} empty styles.\33[0m")
		
		if args.verbosity >= 4: print("Searching for identical styles...")
		removed = 0
		pos = 0
		while '<style:style' in content[pos:]:
			# find style
			pos = content.find('<style:style', pos)
			# get data in matchable format
			data = content[content.find('"', pos + 25) : get_element_end(pos)]
			# search for matching styles
			compare_pos = get_element_end(pos)
			while data in content[compare_pos:]:
				compare_pos = content.rfind('<style:style', compare_pos, content.find(data, compare_pos))
				# remove style
				if content[content.find('"', compare_pos + 25) : get_element_end(compare_pos)] == data:
					content = content.replace(f'style-name="{get_property(compare_pos, "style:name")}"', f'style-name="{get_property(pos, "style:name")}"')
					content = content[:compare_pos] + content[get_element_end(compare_pos):]
					removed += 1
				# next style
				compare_pos = get_element_end(compare_pos)
			pos += 1
		if args.verbosity >= 3 and removed: print(f"\33[92mMerged {removed} duplicate styles.\33[0m")
		
		if args.verbosity >= 4: print("Searching for identical list styles...")
		removed = 0
		pos = 0
		while '<text:list-style' in content[pos:]:
			# find style
			pos = content.find('<text:list-style', pos)
			# get data in matchable format
			data = content[content.find('"', pos + 25) : get_element_end(pos)]
			# search for matching styles
			compare_pos = get_element_end(pos)
			while data in content[compare_pos:]:
				compare_pos = content.rfind('<text:list-style', compare_pos, content.find(data, compare_pos))
				# remove style
				if content[content.find('"', compare_pos + 30) : get_element_end(compare_pos)] == data:
					content = content.replace(f'style-name="{get_property(compare_pos, "style:name")}"', f'style-name="{get_property(pos, "style:name")}"')
					content = content[:compare_pos] + content[get_element_end(compare_pos):]
					removed += 1
				# next style
				compare_pos = get_element_end(compare_pos)
			pos += 1
		if args.verbosity >= 3 and removed: print(f"\33[92mMerged {removed} identical list styles.\33[0m")
		
		# save file
		if content != zipfile.Path(path, 'content.xml').read_text():
			# create new file
			temp_path = path.with_name(path.name + '.tmp')
			if args.verbosity >= 4: print(f"\nSaving changes to {file_name}...")
			try:
				with zipfile.ZipFile(path) as doc:
					with zipfile.ZipFile(temp_path, 'w') as temp_doc:
						temp_doc.writestr('content.xml', content.encode(), compress_type=zipfile.ZIP_DEFLATED)
						for item in doc.infolist():
							if not item.filename in temp_doc.namelist():
								temp_doc.writestr(item, doc.read(item.filename))
				copystat(path, temp_path)
				now = time()
				utime(temp_path, (now, now))
			# error creating file
			except BaseException as error:
				if args.verbosity: print(f"\33[91m{error}\33[0m, save aborted. (File has not been modified.)")
			# rename file
			else:
				shrink = path.stat().st_size - temp_path.stat().st_size
				try:
					if args.disposal == 'trash': send2trash(path)
				except OSError as error:
					if args.verbosity: print(f"\33[91m{error}\33[0m, save aborted.")
				else:
					temp_path.rename(save_path)
					files_cleaned += 1
					if args.verbosity >= 2: print(f"'{save_path}' saved ({shrink}B smaller).")
					total_shrink += shrink
		elif args.verbosity >= 2: print(f"No changes made to {path.name}.")

if args.verbosity >= 2 and len(files) > 1:
	if files_cleaned: print(f"{files_cleaned} out of {len(files)} files cleaned, {total_shrink}B total.")
	else: print("No files were modified.")
