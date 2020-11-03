#!/usr/bin/python

from sys import argv
from sys import exit
import os
import zipfile
sys_arguments = argv
sys_arguments.pop(0)

# basic script info
if not len(sys_arguments):
	print("\033[95m──────── OpenDocument Cleanup Script ────────\033[0m")
	print("This script is used to clean up OpenDocument files, because LibreOffice doesn't seem to do cleanup when saving on it's own.")
	print("To view information on how to use this script, run it with '-h' or '--help' after the script name.")
	exit()

# get arguments
arguments = []
show_help = False
disposal = "none"
remove_fonts = False
remove_language = True
recursive = False
verbosity_string = ""

while len(sys_arguments):
	# argument
	if sys_arguments[0][0] == "-":
		# help
		if sys_arguments[0] == "-h" or sys_arguments[0] == "--help":
			show_help = True
			break
		# value
		elif sys_arguments[0] == "-d" or sys_arguments[0] == "--disposal": disposal = sys_arguments.pop(1)
		elif sys_arguments[0] == "-v" or sys_arguments[0] == "--verbosity": verbosity_string = sys_arguments.pop(1)
		# combinable
		elif not sys_arguments[0][1] == "-":
			for char in sys_arguments[0][1:]:
				if char == "f": remove_fonts = True
				elif char == "l": remove_language = False
				elif char == "r": recursive = True
				else:
					if len(sys_arguments[0]) > 2: print("\033[93mERROR: '-" + char + "' (in '" + sys_arguments[0] + "') is not a valid argument!\033[0m")
					else: print("\033[93mERROR: '-" + char + "' is not a valid argument!\033[0m")
					print("See help (-h or --help) for information on arguments.")
					exit()
		# other
		elif sys_arguments[0] == "--removefonts": remove_fonts = True
		elif sys_arguments[0] == "--keeplanguage": remove_language = False
		elif sys_arguments[0] == "--recursive": recursive = True
		else:
			print("\033[93mERROR: '" + sys_arguments[0] + "' is not a valid argument!\033[0m\nSee help (-h or --help) for information on arguments.")
			exit()
	# positional argument
	else: arguments.append(sys_arguments[0])
	# remove argument
	sys_arguments.pop(0)

del sys_arguments

# help
if show_help:
	print("\n\033[95m──────── OpenDocument Cleanup Usage ────────\033[0m")
	print("This script is used to clean up formatting in OpenDocument files. It is focused on cleaning text documents, but should also be able to clean any OpenDocument types. By default, cleaning removes unused or redundant information in the content of the file.")
	print("The cleanup only affects the content.xml, which has everything that's on the page. That means your styles and such will remain untouched.")
	print("To cleanup a file, run the script with the path to the file you want to cleanup after the script name. You can also put a path to a directory to clean up all the files within it. By default, cleaned-up files will be saved as copies. That way, just in case the script does something that messes up a file, you still have the original. It's a good idea to make sure your files still look right after using the script.")
	print("If an unhandled error occurs when running the script, check on GitHub to see if it's been fixed recently, or if it hasn't, feel free to post the issue.")
	print("\n\033[95m──────── Arguments ────────\033[0m")
	print("-h, --help")
	print("	Display this menu. This argument overrides all other operations, regardless of what other arguments are used.")
	print("-d, --disposal <none|trash|overwrite>")
	print("	What to do with the original file after cleanup.")
	print("	none ────── Leave untouched and create a copy with '-cleanup' at the end of the name. (default)")
	print("	trash ───── Move to trash and replace with the cleaned files.")
	print("	overwrite ─ Overwrite originals. I strongly advise against using this. This script is not perfect, and may make mistakes!")
	print("-f, --removefonts")
	print("	Remove font information (typeface only) from direct formatting information. I highly suggest using this if only one font is used in a given file.")
	print("-l, --keeplanguage")
	print("	Keep language and country information. Not sure what this information is for honestly, but I'm pretty sure it isn't important enough to justify the mess it makes.")
	print("-r, --recursive")
	print("	Also search in subdirectories when given a directory.")
	print("-v, --verbosity <integer from 0-4>")
	print("	How much information will be printed.")
	print("	0 ─ nothing (not recommended)")
	print("	1 ─ errors only")
	print("	2 ─ + error tips & opening/saving")
	print("	3 ─ + data removal stats (default when given directory)")
	print("	4 ─ all available info (default when given file)")
	exit()

# error checking
error = False

# get verbosity
if verbosity_string:
	try: verbosity = int(verbosity_string)
	except: verbosity = -1
	# check if verbosity is valid
	if verbosity < 0 or verbosity > 4:
		error = True
		print("\033[93mERROR: '" + verbosity_string + "' is not a valid verbosity!\033[0m")
		print("Verbosity must be an integer from 0 to 4.\nSee help (-h or --help) for more info on arguments.")
		verbosity = 4
elif len(arguments) and os.path.isdir(arguments[0]): verbosity = 3
else: verbosity = 4

# check disposal method
if not (disposal == "none" or disposal == "trash" or disposal == "overwrite"):
	error = True
	if verbosity >= 1:
		print("\033[93mERROR: '" + disposal + "' is not a valid disposal method!\033[0m")
		if verbosity >= 2: print("Valid disposal methods are: none, trash, overwrite\nSee help (-h or --help) for more info on arguments.")

# check for send2trash
if disposal == "trash":
	try: from send2trash import send2trash
	except:
		error = True
		if verbosity >= 1:
			print("\033[93mERROR: No such module 'send2trash'.\033[0m")
			if verbosity >= 2: print("The python module send2trash is required to use the trash disposal method. Either get send2trash, or use a different disposal method.")

# check path
if not len(arguments):
	error = True
	if verbosity >= 1:
		print("\033[93mERROR: No path given!\033[0m")
		if verbosity >= 2: print("Please name a path to an OpenDocument file or directory to cleanup.")

# exit if there were errors
if error: exit()
del error


# get files

# test if file is a valid document
def is_document(path):
	return zipfile.is_zipfile(path) and zipfile.Path(path, "content.xml").exists()

# given file
if os.path.isfile(arguments[0]):
	if is_document(arguments[0]): files = [arguments[0]]
	# if not a document file
	else:
		if verbosity >= 1:
			print("\033[93mERROR: '" + os.path.basename(arguments[0]) + "' is not a document file!\033[0m")
			if verbosity >= 2: print("This script can only clean up OpenDocument Text files. (These typically end in .odt or .ott.)")
		exit()

# given directory
elif os.path.isdir(arguments[0]):
	if verbosity >= 3: print("Finding documents...")
	files = []
	
	def get_documents(directory):
		global files
		dir_files = []
		for sub_path in os.listdir(directory):
			path = os.path.join(directory, sub_path)
			if os.path.isfile(path) and is_document(path):
				if verbosity >= 4: print("Found file", sub_path)
				dir_files.append(path)
			elif recursive and os.path.isdir(path):
				if verbosity >= 4: print("Found directory", os.path.abspath(path))
				get_documents(path)
		if verbosity >= 3 and recursive and len(dir_files): print("Found", len(dir_files), "files in", os.path.abspath(directory))
		files += sorted(dir_files)
	
	get_documents(arguments[0])
	
	if len(files):
		if verbosity >= 2: print("Total of", len(files), "files found.")
	else:
		if verbosity >= 1:
			if recursive: print("\033[93mNo documents found in '" + os.path.abspath(arguments[0]) + "' or any subdirectories.\033[0m")
			else:
				print("\033[93mNo documents found in '" + os.path.abspath(arguments[0]) + "'.\033[0m")
				if verbosity >= 2: print("Use -r or --recursive to also search subdirectories.")
		exit()

# if given path is not a valid file or directory
else:
	if verbosity >= 1: print("\033[93mERROR: '" + os.path.abspath(arguments[0]) + "' is not a valid file or directory!\033[0m")
	exit()


# cleanup files
files_cleaned = 0
for file_path in files:
	
	file_name = os.path.basename(file_path)
	if verbosity >= 4: print("\nReading \033[95m" + file_name + "\033[0m.")
	elif verbosity >= 3: print("\033[95m" + file_name + "\033[0m")
	
	# get data
	try: content = zipfile.Path(file_path, "content.xml").read_text()
	except:
		if verbosity >= 1: print("\033[93mERROR: '" + file_name + "' is invalid or has disappeared! Skipping file.\033[0m")
	
	# clean file
	else:
		def get_xml_end(xml, search_pos):
			if not xml[search_pos] == '<': search_pos = xml.find('<', 0, search_pos)
			levels = 0
			while True:
				block_end = xml.find('>', search_pos)
				if xml[search_pos + 1] == '/': levels -= 1
				elif not xml[block_end - 1] == '/': levels += 1
				if levels <= 0: return block_end + 1
				search_pos = xml.find('<', search_pos + 1)
		
		
		if verbosity >= 4: print("Beginning content cleanup process...\n")
		
		def get_property(element_pos, name):
			start = content.find(name + '="', element_pos) + len(name) + 2
			return content[start : content.find('"', start)]
		
		def remove_all_elements(elements = []):
			global content
			removed = 0
			while elements[0] in content:
				search_pos = content.find(elements[0])
				end_pos = search_pos + len(elements[0])
				for element in elements[1:]: end_pos = content.index(element, end_pos) + len(element)
				content = content[0:search_pos] + content[end_pos:]
				removed += 1
			return removed
		
		
		if verbosity >= 4: print("Removing irrelevant data...")
		if not content.find(' officeooo:rsid="') == -1:
			removed = remove_all_elements([' officeooo:rsid="', '"'])
			if verbosity >= 3: print("\033[92mRemoved", removed, "officeooo:rsid entries\033[0m")
		if not content.find(' loext:opacity="100%"') == -1:
			removed = remove_all_elements([' loext:opacity="100%"'])
			if verbosity >= 3: print("\033[92mRemoved", removed, "loext:opacity entries\033[0m")
		if remove_fonts:
			if verbosity >= 4: print("Searching for fonts...")
			if not content.find('<style:font-face') == -1:
				removed = remove_all_elements(['<style:font-face', '/>'])
				if verbosity >= 3: print("\033[94mRemoved", removed, "fonts.\033[0m")
			if not content.find(' style:font-name="') == -1:
				removed = remove_all_elements([' style:font-name="', '"'])
				if verbosity >= 3: print("\033[94mRemoved font from", removed, "styles.\033[0m")
		if remove_language:
			if verbosity >= 4: print("Searching for languages and countries...")
			if not content.find('style:language') == -1:
				removed = remove_all_elements([' style:language', '"', '"'])
				if verbosity >= 3: print("\033[92mLanguages removed from direct formatting in", removed, "places.\033[0m")
			if not content.find('style:country') == -1:
				removed = remove_all_elements([' style:country', '"', '"'])
				if verbosity >= 3: print("\033[92mCountries removed from direct formatting in", removed, "places.\033[0m")
		
		if verbosity >= 4: print("Searching for orphan styles...")
		search_pos = 0
		removed = 0
		while '<style:style' in content[search_pos:]:
			# find style
			search_pos = content.find('<style:style', search_pos)
			# check to see if it's used anywhere
			if 'style-name="' + get_property(search_pos, 'style:name') in content: search_pos += 1
			else:
				# remove style
				content = content[:search_pos] + content[get_xml_end(content, search_pos):]
				removed += 1
		if verbosity >= 3 and removed: print("\033[92mRemoved", removed, "orphan styles.\033[0m")
		
		if verbosity >= 4: print("Searching for orphan list styles...")
		search_pos = 0
		removed = 0
		while '<text:list-style' in content[search_pos:]:
			# find style
			search_pos = content.find('<text:list-style', search_pos)
			# check to see if it's used anywhere
			if 'style-name="' + get_property(search_pos, 'style:name') in content: search_pos += 1
			else:
				# remove style
				content = content[:search_pos] + content[get_xml_end(content, search_pos):]
				removed += 1
		if verbosity >= 3 and removed: print("\033[92mRemoved", removed, "orphan lists styles.\033[0m")
		
		if verbosity >= 4: print("Searching for duplicate styles...")
		search_pos = 0
		removed = 0
		while '<style:style' in content[search_pos:]:
			# find style
			search_pos = content.find('<style:style', search_pos)
			# get data in matchable format
			style_data = content[content.find('"', search_pos + 25) : get_xml_end(content, search_pos)]
			# search for matching style
			compare_pos = search_pos + 1
			while '<style:style' in content[compare_pos:]:
				compare_pos = content.find('<style:style', compare_pos)
				# remove style
				if style_data == content[content.find('"', compare_pos + 25) : get_xml_end(content, compare_pos)]:
					content = content.replace('style-name="' + get_property(compare_pos, 'style:name'), 'style-name="' + get_property(search_pos, 'style:name'))
					content = content[:compare_pos] + content[get_xml_end(content, compare_pos):]
					removed += 1
				# next style
				else: compare_pos += 1
			search_pos += 1
		if verbosity >= 3 and removed: print("\033[92mRemoved", removed, "duplicate styles.\033[0m")
		
		if verbosity >= 4: print("Searching for duplicate list styles...")
		search_pos = 0
		removed = 0
		while '<text:list-style' in content[search_pos:]:
			# find style
			search_pos = content.find('<text:list-style', search_pos)
			# get data in matchable format
			style_data = content[content.find('"', search_pos + 30) : get_xml_end(content, search_pos)]
			# search for matching style
			compare_pos = search_pos + 1
			while '<text:list-style' in content[compare_pos:]:
				compare_pos = content.find('<text:list-style', compare_pos)
				# remove style
				if style_data == content[content.find('"', compare_pos + 30) : get_xml_end(content, compare_pos)]:
					content = content.replace('style-name="' + get_property(compare_pos, 'style:name'), 'style-name="' + get_property(search_pos, 'style:name'))
					content = content[:compare_pos] + content[get_xml_end(content, compare_pos):]
					removed += 1
				# next style
				else: compare_pos += 1
			search_pos += 1
		if verbosity >= 3 and removed: print("\033[92mRemoved", removed, "duplicate list styles.\033[0m")
		
		search_pos = 0
		empty = False
		while '<style:style' in content[search_pos:] and not empty:
			# find style
			search_pos = content.find('<style:style', search_pos)
			# check properties
			if '<style:text-properties/>' in content[search_pos : get_xml_end(content, search_pos)]: empty = True
			search_pos += 1
		if empty:
			if verbosity >= 4: print("\033[93mFound an empty style leftover.\033[0m LibreOffice will clean these up on it's own when saving the file.")
			elif verbosity >= 3: print("\033[93mFound an empty style leftover.\033[0m")
		
		# save file
		if content == zipfile.Path(file_path, "content.xml").read_text():
			if verbosity >= 2: print("No changes made to '" + file_name + "'. Skipping saving process.")
		else:
			if verbosity >= 4: print("\nUpdating " + file_name + "...")
			try:
				with zipfile.ZipFile(file_path) as doc:
					with zipfile.ZipFile(file_path + ".cleanup", "w") as temp_doc:
						temp_doc.writestr("content.xml", content.encode(), compress_type=zipfile.ZIP_DEFLATED)
						for item in doc.infolist():
							if not temp_doc.namelist().count(item.filename):
								temp_doc.writestr(item, doc.read(item.filename))
			except:
				if verbosity >= 1: print("\033[93mSaving failed! File has not been modified.\033[0m")
			# rename file
			else:
				new_file_path = file_path
				if disposal == "overwrite": os.remove(file_path)
				elif disposal == "trash": send2trash(file_path)
				elif disposal == "none":
					if "." in file_name: new_file_path = file_path[:file_path.rfind(".")] + "-cleaned" + file_path[file_path.rfind("."):]
					else: new_file_path += "-cleaned"
					file_name = os.path.basename(new_file_path)
				os.rename(file_path + ".cleanup", new_file_path)
				
				if verbosity >= 2: print("\033[92m" + file_name + " saved.\033[0m")
				files_cleaned += 1

if len(files) > 1: print(files_cleaned, "out of", len(files), "files cleaned.")
