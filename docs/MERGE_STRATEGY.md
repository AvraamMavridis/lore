# Lore Merge Strategy: Append-Only Reasoning

## Overview

Lore implements an **append-only merge strategy** for handling conflicts when multiple developers update reasoning for the same module on different branches. This ensures no reasoning is lost during merges—both branches' perspectives are preserved.

## Problem Statement

When two developers work on different branches and both record reasoning for the same file:
- **Developer A** on `feature/auth` records: "Refactored JWT validation"
- **Developer B** on `feature/perf` records: "Optimized token caching"

Git would normally create a merge conflict in `.lore/index.json`. Traditional merge strategies would require choosing one version, losing valuable context from the other branch.

## Solution: Append-Only Merge

Instead of choosing one entry over another, Lore's merge strategy **preserves both entries**. This aligns with Lore's core philosophy: preserve all reasoning context.

### How It Works

1. **Git Merge Driver**: When `.lore/index.json` conflicts during a merge, a custom merge driver is invoked
2. **Index Merging**: The driver intelligently combines entry IDs from both branches
3. **Deduplication**: If both branches reference the same entry ID, it only appears once
4. **Result**: The merged index contains all unique entry IDs from both branches

### Example

**Before Merge (Branch A):**
```json
{
  "files": {
    "src/auth.rs": ["entry-a1", "entry-a2"]
  },
  "entry_count": 2
}
```

**Before Merge (Branch B):**
```json
{
  "files": {
    "src/auth.rs": ["entry-b1"]
  },
  "entry_count": 1
}
```

**After Merge (Automatic):**
```json
{
  "files": {
    "src/auth.rs": ["entry-a1", "entry-a2", "entry-b1"]
  },
  "entry_count": 3
}
```

All reasoning from both branches is preserved. Developers can see the complete decision history:

```bash
lore explain src/auth.rs --all
```

This shows:
- JWT validation refactoring reasoning (Branch A)
- Token caching optimization reasoning (Branch B)
- Evolution of thinking across different implementation approaches

## Setup Instructions

### Enable the Merge Driver

The merge driver is automatically configured when you clone/pull from the repository. To manually enable it:

```bash
# Configure the merge driver
git config merge.lore.name "Lore append-only merge driver"
git config merge.lore.driver "./scripts/lore-merge-driver.sh %O %A %B"

# The .gitattributes file (already in repo) defines which files use the driver
cat .gitattributes
```

### What Happens During a Merge

1. Git detects a conflict in `.lore/index.json`
2. Instead of creating a conflict marker, Git invokes the `lore-merge-driver`
3. The driver:
   - Reads both branch versions
   - Combines entry IDs (append-only)
   - Deduplicates any shared entries
   - Writes the merged result
4. Merge completes without manual conflict resolution

## Viewing Merged Reasoning

After a merge with reasoning conflicts, view all perspectives:

```bash
# Show most recent reasoning (default)
lore explain src/auth.rs

# Show ALL reasoning from both branches
lore explain src/auth.rs --all

# See which branch each entry came from
lore explain src/auth.rs --all --json | jq '.[] | {agent_id, intent, timestamp}'
```

### Example Output

```
Lore for: src/auth.rs
══════════════════════════════════════════════════════════════

Agent: claude-code (feature/auth) │ 2026-01-14 10:00:00 UTC
Commit: a1b2c3d4

Intent:
Refactored JWT validation to handle refresh tokens

Reasoning:
  The existing implementation only supported access tokens.
  I added refresh token support by creating separate validation paths...

──────────────────────────────────────────────────────────────

Agent: claude-code (feature/perf) │ 2026-01-14 11:30:00 UTC
Commit: e5f6g7h8

Intent:
Optimized token caching to reduce validation overhead

Reasoning:
  Token validation was a bottleneck. I implemented caching with TTL...
```

## Benefits

### No Lost Context
Every decision from every branch is preserved, giving future developers complete historical context.

### Reduced Conflicts
No manual conflict resolution needed for `.lore/index.json`—Git handles it automatically.

### Branch Equality
Both branches' reasoning is treated equally—neither overwrites the other.

### Evolutionary Understanding
View how different approaches to the same problem emerged and diverged across branches.

## Technical Details

### Merge Driver Script

The merge driver (`scripts/lore-merge-driver.sh`) is a shell script that:
1. Receives three parameters: BASE (common ancestor), OURS (current branch), THEIRS (incoming branch)
2. Parses both index files as JSON
3. Combines entry IDs using a set (for deduplication)
4. Writes the merged result back to OURS
5. Exits with status 0 on success

```bash
./scripts/lore-merge-driver.sh <BASE> <OURS> <THEIRS>
```

### .gitattributes Configuration

```
.lore/index.json merge=lore
.lore/entries/*.json merge=lore
```

This tells Git to use the `lore` merge driver for these files.

### Deduplication Algorithm

The merge uses a HashSet to ensure each entry ID appears only once:
- If both branches have the same entry ID, it's included once
- If branches have different entries, all are included
- Result is sorted for consistent output

## Edge Cases

### Same Entry ID in Both Branches
If both branches reference the same entry ID (rare, but possible):
```
Branch A: src/auth.rs → [entry-1, entry-2]
Branch B: src/auth.rs → [entry-1, entry-3]
Result:   src/auth.rs → [entry-1, entry-2, entry-3]
```
The duplicate `entry-1` is deduplicated.

### One Branch Has No Entries
```
Branch A: src/auth.rs → [entry-1]
Branch B: (no entries for src/auth.rs)
Result:   src/auth.rs → [entry-1]
```
Works as expected—entries from one branch are preserved.

### Multiple Files Modified
Each file's entries are merged independently:
```
Branch A: src/auth.rs → [entry-a1], src/db.rs → [entry-d1]
Branch B: src/auth.rs → [entry-a2], src/cache.rs → [entry-c1]
Result:   src/auth.rs → [entry-a1, entry-a2]
          src/db.rs → [entry-d1]
          src/cache.rs → [entry-c1]
```

## Limitations & Future Enhancements

### Current Limitations
- Merge driver only works for `index.json`—individual entry files (`.lore/entries/*.json`) follow Git's default merge behavior
- Python 3 required for the merge driver script

### Potential Enhancements
1. **Merge Visualization UI**: Web interface showing side-by-side reasoning from different branches
2. **Entry Merging**: Automatically merge individual entry files when conflicts occur
3. **Conflict Statistics**: Report how many entries were merged per file
4. **Merge History**: Track which branches contributed which entries
5. **Agent-Aware Merging**: Different merge strategies based on agent type (human vs AI)

## Troubleshooting

### Merge Driver Not Working

1. **Check Git is using the driver:**
   ```bash
   git config --list | grep merge.lore
   ```

2. **Verify the script is executable:**
   ```bash
   ls -la scripts/lore-merge-driver.sh
   # Should show -rwxr-xr-x
   ```

3. **Check .gitattributes:**
   ```bash
   cat .gitattributes
   # Should contain: .lore/index.json merge=lore
   ```

4. **Test manually:**
   ```bash
   ./scripts/lore-merge-driver.sh <base-file> <ours-file> <theirs-file>
   ```

### Still Getting Merge Conflicts

- The driver still failed (check Git logs)
- Use `lore explain --all` to view all reasoning regardless
- Manually inspect `.lore/entries/` to verify both branches' entries exist

## Related Commands

- `lore explain <file> --all` - View all reasoning versions
- `lore explain <file> --json` - JSON output for programmatic access
- `lore search <query>` - Search across all reasoning
- `lore status` - See which files have multiple versions

## References

- Git Merge Driver Documentation: `git help attributes`
- Lore Index Format: `.lore/index.json`
- Lore Entry Format: `.lore/entries/<uuid>.json`
