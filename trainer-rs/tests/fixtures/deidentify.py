#!/usr/bin/env python3
"""
Produces a de-identified export fixture from a real Trainer export.

The fixture's purpose is proving SERIALIZATION FORMAT parity between the C#
implementation and the Rust port. It does not need real content, so activity
type names, units, note text, location names, and coordinates are replaced.

Everything structural is preserved verbatim:
  - key order within every object
  - field presence and absence (notes / durationSeconds / knownLocationId are
    omitted, never null, in the export format)
  - the empty-string vs non-empty distinction in notes
  - all ids, including the large signed knownLocation ids
  - all timestamps, including the non-standard hour-only UTC offsets (-08/-07)
  - week bucket keys, including the year-boundary bucket 2026.01
  - coordinate values keep 7-decimal precision so double formatting is exercised
  - minified output with no whitespace, matching WriteIndented = false

Usage:
    python3 deidentify.py <real-export.json> <output.json>
"""

import json
import sys

# Generic replacements. Index-stable so repeated runs produce identical output.
TYPE_NAMES = [
    "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel",
    "India", "Juliett", "Kilo", "Lima", "Mike", "November", "Oscar", "Papa",
]
UNITS = ["oz", "ct", "min", "rep", "mi", "g", "ml", "set"]
NOTE_TEXTS = [
    "Note one", "Note two", "A slightly longer note value", "Third note",
    "Note with trailing detail", "Short", "Another note", "Final note text",
]
LOCATION_NAMES = [
    "Location A", "Location B", "Location C", "Location D", "Location E",
    "Location F", "Location G", "Location H", "Location I", "Location J",
    "New Location 1",
]


def deidentify(src: dict) -> dict:
    out = {}

    # --- activities: preserve bucket order, field order, presence, timestamps ---
    activities = {}
    note_i = 0
    for week_key, entries in src["activities"].items():
        new_entries = []
        for a in entries:
            new = {}
            for k, v in a.items():  # preserve original key order
                if k == "notes":
                    # Preserve the empty vs non-empty distinction exactly.
                    if v == "":
                        new[k] = ""
                    else:
                        new[k] = NOTE_TEXTS[note_i % len(NOTE_TEXTS)]
                        note_i += 1
                else:
                    new[k] = v
            new_entries.append(new)
        activities[week_key] = new_entries
    out["activities"] = activities

    # --- activityTypes: replace name and unit, keep everything else ---
    types = []
    for i, t in enumerate(src["activityTypes"]):
        new = {}
        for k, v in t.items():
            if k == "name":
                new[k] = TYPE_NAMES[i % len(TYPE_NAMES)]
            elif k == "unit":
                new[k] = UNITS[i % len(UNITS)]
            else:
                new[k] = v
        types.append(new)
    out["activityTypes"] = types

    # --- knownLocations: replace name and coordinates, KEEP ids ---
    # Ids are preserved because activities reference them, and because they are
    # produced by HashCode.Combine, which .NET seeds randomly per process — they
    # are opaque, non-invertible, and not reproducible by any implementation.
    locations = []
    for i, loc in enumerate(src["knownLocations"]):
        new = {}
        for k, v in loc.items():
            if k == "name":
                new[k] = LOCATION_NAMES[i % len(LOCATION_NAMES)]
            elif k == "latitude":
                new[k] = round(10.0 + i * 1.1111111, 7)
            elif k == "longitude":
                new[k] = round(-20.0 - i * 1.1111111, 7)
            else:
                new[k] = v
        locations.append(new)
    out["knownLocations"] = locations

    out["exportDate"] = src["exportDate"]
    return out


def deidentify_snapshot(src: dict) -> dict:
    """
    De-identifies a raw IndexedDB snapshot. Unlike the export, storage writes every
    optional field explicitly, so nulls must be preserved as nulls and empty strings
    as empty strings — the three notes states (null / "" / text) are distinct and are
    serialized differently by the two configurations.

    Also preserves the value-representation metadata, which is the whole point of this
    fixture: 33 entries are structured-cloned Arrays and activityNextId is a bare Number.
    """
    out = {k: src[k] for k in ("dbName", "dbVersion", "objectStore", "keyCount")}

    entries = {}
    note_i = 0
    type_i = 0
    loc_i = 0

    for key, entry in src["entries"].items():
        new_entry = {k: entry[k] for k in ("typeofValue", "isArray", "constructor")}
        value = entry["value"]

        if key.startswith("activities-"):
            new_rows = []
            for a in value:
                row = {}
                for k, v in a.items():
                    if k == "notes":
                        if v is None:
                            row[k] = None          # null stays null
                        elif v == "":
                            row[k] = ""            # empty string stays empty string
                        else:
                            row[k] = NOTE_TEXTS[note_i % len(NOTE_TEXTS)]
                            note_i += 1
                    else:
                        row[k] = v
                new_rows.append(row)
            new_entry["value"] = new_rows

        elif key == "activityTypes":
            new_types = []
            for t in value:
                row = {}
                for k, v in t.items():
                    if k == "name":
                        row[k] = TYPE_NAMES[type_i % len(TYPE_NAMES)]
                    elif k == "unit":
                        row[k] = None if v is None else UNITS[type_i % len(UNITS)]
                    else:
                        row[k] = v
                new_types.append(row)
                type_i += 1
            new_entry["value"] = new_types

        elif key == "knownLocations":
            new_locs = []
            for loc in value:
                row = {}
                for k, v in loc.items():
                    if k == "name":
                        row[k] = LOCATION_NAMES[loc_i % len(LOCATION_NAMES)]
                    elif k == "latitude":
                        row[k] = round(10.0 + loc_i * 1.1111111, 7)
                    elif k == "longitude":
                        row[k] = round(-20.0 - loc_i * 1.1111111, 7)
                    else:
                        row[k] = v
                new_locs.append(row)
                loc_i += 1
            new_entry["value"] = new_locs

        else:
            # activityNextId and anything else: scalar, no personal content.
            new_entry["value"] = value

        entries[key] = new_entry

    out["entries"] = entries
    out["localStorage"] = dict(src.get("localStorage", {}))
    return out


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__)
        return 2

    with open(sys.argv[1], encoding="utf-8") as f:
        src = json.load(f)

    result = deidentify_snapshot(src) if "entries" in src else deidentify(src)

    # Match System.Text.Json with WriteIndented = false: no whitespace at all.
    with open(sys.argv[2], "w", encoding="utf-8") as f:
        json.dump(result, f, separators=(",", ":"), ensure_ascii=False)

    return 0


if __name__ == "__main__":
    sys.exit(main())
