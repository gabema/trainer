---
name: ugrep-project-search
description: Use ugrep and ugrep-indexer to index and search the current project for matching text with fast fuzzy and case-insensitive pattern matching. Use when searching for any text (symbols, strings, or identifiers) across the codebase or when the agent needs project-wide matches while performing a task.
---

# ugrep Project Search

## When to use this skill

Use this skill to:
- Quickly search all files in the current project using `ugrep`
- Build and refresh a local search index with `ugrep-indexer`
- Run case-insensitive and fuzzy (approximate) text searches

This skill assumes the **project root** is the directory you want to search (typically the git repository root or the workspace folder).

## Prerequisites

1. **Ensure ugrep is installed** (includes `ug` and `ugrep-indexer`).

   Install it using your OS/package manager of choice, for example:

   ```bash
   # macOS (Homebrew)
   brew install ugrep

   # Debian/Ubuntu
   sudo apt-get install ugrep

   # Fedora
   sudo dnf install ugrep

   # Arch Linux
   sudo pacman -S ugrep
   ```

   Or see the official installation instructions for your platform (`ugrep` documentation or package repository).

2. **Run all commands from the project root** unless noted otherwise, e.g.:

   ```bash
   cd /path/to/your/project
   ```

## Indexing the project

Use `ugrep-indexer` once (and periodically) to build or refresh an index for faster recursive searches:

```bash
cd /path/to/your/project

# Create or refresh the index for the whole project
ugrep-indexer -I -z .
```

- **`-I`**: ignore binary files to keep the index small and focused on text.
- **`-z`**: include archives and compressed files when indexing (optional; omit if not needed).

You can safely re-run `ugrep-indexer` at any time; it will incrementally update the index. Use `-f` (`--force`) if you want to force a full reindex:

```bash
ugrep-indexer -I -z -f .
```

To delete the index (if you need a clean start):

```bash
ugrep-indexer -d .
```

## Basic indexed search

After indexing, run project-wide searches using the index:

```bash
cd /path/to/your/project

# Exact match search using the index
ugrep --index "exact pattern"
```

You can also use the `ug` alias (interactive TUI) with the index:

```bash
ug --index "exact pattern"
```

## Case-insensitive searches

Use `-i` / `--ignore-case` for case-insensitive matching:

```bash
# Case-insensitive search
ugrep --index -i "pattern"

# Case-insensitive search limited to C# files
ugrep --index -i -G '*.cs' "pattern"

# Case-insensitive search using the interactive TUI
ug --index -i "pattern"
```

If you prefer smart case (only ignore case when the pattern is all lowercase), use `-j`:

```bash
ugrep --index -j "pattern"
```

## Fuzzy / approximate searches

Use `-Z` / `--fuzzy` for fuzzy (approximate) matching, which tolerates small typos and differences:

```bash
# Fuzzy search with default tolerance
ugrep --index -Z "pattren"

# Fuzzy, case-insensitive search
ugrep --index -i -Z "pattren"

# Fuzzy search in the TUI
ug --index -Z "pattren"
```

For more control over fuzzy matching (e.g., limiting allowed insertions, deletions, substitutions), consult the built-in help:

```bash
ug --help fuzzy
```

## Combining filters

You can combine indexed, case-insensitive, and fuzzy searching with file globs and other filters:

```bash
# Fuzzy, case-insensitive search in C# source files only
ugrep --index -i -Z -G '*.cs' "pattren"

# Case-insensitive search limited to a subdirectory
ugrep --index -i -R Trainer "pattern"
```

Commonly useful options:
- **`--line-number`** / `-n`: show line numbers (on by default with `ugrep` in many builds).
- **`-R <dir>`**: restrict search to a specific subdirectory.
- **`-G 'glob'`**: only search files matching a glob pattern (e.g., `*.cs`, `*.md`).

## Examples

- **Find usages of a method regardless of case**:

  ```bash
  ugrep --index -i "trainermodelbuilder"
  ```

- **Search for a configuration key with possible typos**:

  ```bash
  ugrep --index -i -Z "ConnectionStrng"
  ```

- **Search only in C# code under the `Trainer` folder**:

  ```bash
  ugrep --index -i -G '*.cs' -R Trainer "GetTrainingConfig"
  ```

Follow this sequence when using the skill:
1. From the project root, ensure the index exists or is refreshed with `ugrep-indexer`.
2. Use `ugrep --index` (or `ug --index`) with `-i` and/or `-Z` depending on whether you need case-insensitive or fuzzy matching.
3. Narrow the search with `-G` and `-R` as needed to focus on relevant files and directories.

