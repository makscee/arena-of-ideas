#!/usr/bin/env python3
"""Load world assets from arena-of-ideas-world repo into local SpacetimeDB."""

import os
import re
import subprocess
import sys

WORLD_PATH = sys.argv[1] if len(sys.argv) > 1 else "/Users/admin/Documents/GitHub/arena-of-ideas-world/assets/nodes"
SERVER = sys.argv[2] if len(sys.argv) > 2 else "local"
DB = sys.argv[3] if len(sys.argv) > 3 else "aoi-local"

def sql(query):
    result = subprocess.run(
        ["spacetime", "sql", DB, query, "-s", SERVER],
        capture_output=True, text=True, timeout=30
    )
    if result.returncode != 0 and "Error" in result.stderr:
        print(f"  SQL ERROR: {result.stderr.strip()[:200]}")
        return False
    return True

def sql_escape(s):
    """Escape a string for SQL single-quoted literals."""
    return s.replace("'", "''")

def ron_unescape_string(ron_str):
    """Convert a RON string literal to its actual value.
    RON: "\"Paladins\"" -> "Paladins"
    RON: "(1, 2)" -> (1, 2)
    """
    s = ron_str.strip()
    if s.startswith('"') and s.endswith('"'):
        # It's a RON string literal - remove outer quotes and unescape
        inner = s[1:-1]
        inner = inner.replace('\\"', '"')
        inner = inner.replace('\\\\', '\\')
        return inner
    return s

def parse_node_file(filepath):
    """Parse a RON node asset file: (data, owner, rating)"""
    with open(filepath) as f:
        content = f.read().strip()

    # Match: (data, owner_int, rating_int)
    # The data can be complex RON, so we find owner and rating from the end
    m = re.match(r'^\((.*),\s*(\d+),\s*(\d+)\)$', content, re.DOTALL)
    if m:
        raw_data = m.group(1).strip()
        owner = int(m.group(2))
        rating = int(m.group(3))
        data = ron_unescape_string(raw_data)
        return data, owner, rating
    return None, None, None

# Clear existing data first
print("Clearing existing data...")
sql("DELETE FROM node_links WHERE id > 0")
sql("DELETE FROM nodes_world WHERE id > 3")  # Keep NArena (id=3)

# Load nodes
node_count = 0
for kind in sorted(os.listdir(WORLD_PATH)):
    kind_path = os.path.join(WORLD_PATH, kind)
    if not os.path.isdir(kind_path):
        continue

    files = [f for f in os.listdir(kind_path) if f.endswith(".ron")]
    print(f"Loading {len(files)} {kind} nodes...")

    for fname in files:
        node_id = fname.replace(".ron", "")
        filepath = os.path.join(kind_path, fname)

        data, owner, rating = parse_node_file(filepath)
        if data is None:
            print(f"  WARN: Could not parse {filepath}")
            continue

        escaped = sql_escape(data)
        q = f"INSERT INTO nodes_world (id, owner, kind, data, rating) VALUES ({node_id}, {owner}, '{kind}', '{escaped}', {rating})"
        if sql(q):
            node_count += 1
        else:
            print(f"  Failed: {fname}")

print(f"\nLoaded {node_count} nodes")

# Load links
links_file = os.path.join(WORLD_PATH, "links.ron")
if os.path.exists(links_file):
    print("Loading links...")
    with open(links_file) as f:
        content = f.read()

    tuples = re.findall(
        r'\((\d+),\s*(\d+),\s*"(\w+)",\s*"(\w+)",\s*\d+,\s*\d+\)',
        content
    )

    link_count = 0
    for i, (parent, child, pkind, ckind) in enumerate(tuples):
        link_id = 9000000000000000 + i
        q = f"INSERT INTO node_links (id, parent, child, parent_kind, child_kind) VALUES ({link_id}, {parent}, {child}, '{pkind}', '{ckind}')"
        if sql(q):
            link_count += 1

    print(f"Loaded {link_count} links")

print("Done!")
