#!/bin/bash
# Git merge driver for Lore index files
# This script is called by Git during merges when conflicts occur in .lore/index.json
# Git passes three arguments: BASE, OURS, THEIRS
# We need to merge them and write the result back to OURS

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <BASE> <OURS> <THEIRS>"
    echo "Git merge driver for Lore index files"
    exit 1
fi

BASE="$1"
OURS="$2"
THEIRS="$3"

# Use Python to merge the JSON files with append-only strategy
python3 << 'PYTHON_EOF'
import json
import sys
from pathlib import Path

ours_path = sys.argv[1]
theirs_path = sys.argv[2]

def merge_indexes(ours_path, theirs_path):
    """Merge two Lore index.json files using append-only strategy"""
    try:
        with open(ours_path, 'r') as f:
            ours = json.load(f)
    except:
        ours = {"files": {}, "entry_count": 0}

    try:
        with open(theirs_path, 'r') as f:
            theirs = json.load(f)
    except:
        theirs = {"files": {}, "entry_count": 0}

    # Merge the files, preserving all entries (append-only strategy)
    merged_files = {}
    all_files = set(ours.get("files", {}).keys()) | set(theirs.get("files", {}).keys())

    for file_path in all_files:
        # Use a set to deduplicate entry IDs from both branches
        entry_ids = set()

        if file_path in ours.get("files", {}):
            entry_ids.update(ours["files"][file_path])

        if file_path in theirs.get("files", {}):
            entry_ids.update(theirs["files"][file_path])

        # Sort for consistent output
        merged_files[file_path] = sorted(list(entry_ids))

    merged = {
        "files": merged_files,
        "entry_count": sum(len(ids) for ids in merged_files.values())
    }

    return merged

# Perform the merge
merged = merge_indexes(ours_path, theirs_path)

# Write back to OURS
with open(ours_path, 'w') as f:
    json.dump(merged, f, indent=2)
    f.write('\n')

print(f"✓ Merged Lore index.json: {merged['entry_count']} total entries across {len(merged['files'])} files", file=sys.stderr)
sys.exit(0)
PYTHON_EOF

exit $?
