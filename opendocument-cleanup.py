#!/usr/bin/python

import sys
sys_arguments = sys.argv
sys_arguments.pop(0)

if len(sys_arguments):
	
	arguments = []
	
	show_help = False
	remove_fonts = False
	remove_language = False
	overwrite = False
	
	while len(sys_arguments):
		if sys_arguments[0] == "-h" or sys_arguments[0] == "--help":
			show_help = True
			break
		if sys_arguments[0] == "-f" or sys_arguments[0] == "--fonts":
			remove_fonts = True
			sys_arguments.pop(0)
		if sys_arguments[0] == "-l" or sys_arguments[0] == "--language":
			remove_language = True
			sys_arguments.pop(0)
		if sys_arguments[0] == "-o" or sys_arguments[0] == "--overwrite":
			overwrite = True
			sys_arguments.pop(0)
		else:
			arguments.append(sys_arguments.pop(0))
	
	del sys_arguments
	
	if show_help:
		print("\n\033[95m──────── OpenDocument Cleanup Usage ────────\033[0m")
		print("This script is used to clean up formatting in OpenDocument Text (.odt) files. By default, this includes unused or redundant information in the content of the document.")
		print("The document cleanup only affects the content of the document, being everything that's on the page. That means styles will remain untouched.")
		print("To cleanup a document, run the script with the path to the document you want to cleanup after the script name. By default the cleaned-up document will be saved as a copy. That way, just in case the script does something that messes up your document, you still have the original. It's a good idea to make sure the document still looks right after using the script.")
		print("\n\033[95m──────── Arguments ────────\033[0m")
		print("-h, --help")
		print("	Display this menu. This argument overrides all other operations, regardless of what other arguments are used.")
		print("-f, --fonts")
		print("	Remove font information (typeface only) from direct formatting information. I highly suggest you use this if only one font is used for the entire document.")
		print("-l, --language")
		print("	Remove language and country information. I highly suggest you use this if only one font is used for the entire document.")
		print("-o, --overwrite")
		print("	Overwrite original document instead of creating a copy.")
		print("	You should probably only do this if you have another way to revert your document, just in case this script messes something up.")
	else:
		import os
		import zipfile
		
		print("Reading", arguments[0])
		content = ""
		with zipfile.ZipFile(arguments[0]) as doc:
			with doc.open("content.xml") as doc_content:
				content = doc_content.read().decode()
		
		
		print("\033[95mBeginning content cleanup process...\033[0m\n")
		
		def get_property(element_pos, name):
			start = content.find(name + '="', element_pos) + len(name) + 2
			return content[start : content.find('"', start)]
		
		def remove_all_elements(elements = []):
			global content
			removed = 0
			while True:
				try:
					search_pos = content.index(elements[0])
					end_pos = search_pos + len(elements[0])
					for element in elements[1:]: end_pos = content.index(element, end_pos) + len(element)
				except:
					return removed
				content = content[0:search_pos] + content[end_pos:]
				removed += 1
		
		
		print("Taking out the trash...")
		if not content.find(' officeooo:rsid="') == -1: print("\033[92mRemoved", remove_all_elements([' officeooo:rsid="', '"']), "officeooo:rsid entries\033[0m")
		if not content.find(' loext:opacity="100%"') == -1: print("\033[92mRemoved", remove_all_elements([' loext:opacity="100%"']), "loext:opacity entries\033[0m")
		if remove_fonts:
			print("Searching for fonts...")
			if not content.find('<style:font-face') == -1: print("\033[94mRemoved", remove_all_elements(['<style:font-face', '/>']), "fonts.\033[0m")
			if not content.find(' style:font-name="') == -1: print("\033[94mRemoved font from", remove_all_elements([' style:font-name="', '"']), "styles.\033[0m")
		if remove_language:
			print("Searching for languages and countries...")
			if not content.find('style:language') == -1: print("\033[92mLanguages removed from direct formatting in", remove_all_elements([' style:language', '"', '"']), "places.\033[0m")
			if not content.find('style:country') == -1: print("\033[92mCountries removed from direct formatting in", remove_all_elements([' style:country', '"', '"']), "places.\033[0m")
		
		print("Searching for orphan styles...")
		search_pos = 0
		removed = 0
		while True:
			# find style
			try: search_pos = content.index('<style:style', search_pos)
			except: break
			# check to see if it's used anywhere
			try: content.index('text:style-name="' + get_property(search_pos, 'style:name'))
			# remove style
			except:
				content = content[:search_pos] + content[content.index('</style:style>', search_pos) + 14:]
				removed += 1
			# next style
			else: search_pos += 1
		if removed: print("\033[92mRemoved", removed, "orphan styles.\033[0m")
		
		print("Searching for orphan list styles...")
		search_pos = 0
		removed = 0
		while True:
			# find style
			try: search_pos = content.index('<text:list-style', search_pos)
			except: break
			# check to see if it's used anywhere
			try: content.index('text:style-name="' + get_property(search_pos, 'style:name'))
			# remove style
			except:
				content = content[:search_pos] + content[content.index('</text:list-style>', search_pos) + 18:]
				removed += 1
			# next style
			else: search_pos += 1
		if removed: print("\033[92mRemoved", removed, "orphan lists styles!\033[0m")
		
		print("Searching for duplicate styles...")
		search_pos = 0
		removed = 0
		while True:
			# find style
			try: search_pos = content.index('<style:style', search_pos)
			except: break
			# get data in matchable format
			style_data = content[content.find('"', search_pos + 25) : content.find('</style:style>', search_pos)]
			# search for matching style
			compare_pos = search_pos + 1
			while True:
				try: compare_pos = content.index('<style:style', compare_pos)
				except: break
				# remove style
				if style_data == content[content.find('"', compare_pos + 25) : content.find('</style:style>', compare_pos)]:
					content = content.replace('text:style-name="' + get_property(compare_pos, 'style:name'), 'text:style-name="' + get_property(search_pos, 'style:name'))
					content = content[:compare_pos] + content[content.index('</style:style>', compare_pos) + 14:]
					removed += 1
				# next style
				else: compare_pos += 1
			search_pos += 1
		if removed: print("\033[92mRemoved", removed, "duplicate styles.\033[0m")
		
		
		print("\nUpdating document...")
		try:
			with zipfile.ZipFile(arguments[0]) as doc:
				with zipfile.ZipFile(arguments[0] + ".tmp", "w") as temp_doc:
					temp_doc.writestr("content.xml", content.encode())
					for item in doc.infolist():
						if not temp_doc.namelist().count(item.filename):
							temp_doc.writestr(item, doc.read(item.filename))
		except:
			print("\033[93mSaving failed! Document has not been modified.\033[0m")
		else:
			if overwrite:
				os.remove(arguments[0])
				os.rename(arguments[0] + ".tmp", arguments[0])
			else: os.rename(arguments[0] + ".tmp", arguments[0].replace(".odt", "-cleaned.odt"))
			print("\033[92mDocument saved.\033[0m")
			
			search_pos = 0
			empty = 0
			while True:
				# find style
				try: search_pos = content.index('<style:style', search_pos)
				except: break
				# check properties
				if not content.find('<style:text-properties/>', search_pos, content.index('</style:style>', search_pos)) == -1: empty += 1
				search_pos += 1
			if empty: print("Found", empty, "empty styles leftover. LibreOffice will clean these up on it's own when saving the document.")
else:
	print("\033[95m──────── OpenDocument Cleanup Script ────────\033[0m")
	print("This script is used to clean up OpenDocument Text (.odt) files, because LibreOffice doesn't seem to do any cleanup when saving on it's own.")
	print("To view information on how to use this script, use \"-h\" or \"--help\" after the command used to run it.")
