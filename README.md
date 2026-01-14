<p align="center">
  <img src="logo.png" alt="Lore" width="200">
</p>

# Lore

A reasoning engine for code — stores the "why" behind changes.

Lore creates a high-context repository where AI agents don't just read your code—they read the minds of previous developers and agents who worked on it.

## What is Lore?

While Git tells you **who** changed code and **when**, Lore tells you **why**. It stores:

- **Intent**: Brief description of what you were trying to accomplish
- **Reasoning Trace**: Full chain-of-thought, which can be thousands of words
- **Rejected Alternatives**: What you tried but didn't work (and why)
- **Tags**: Categorization for easy searching

All entries are cryptographically linked to file content hashes and optionally to Git commits.

## Why Lore?

| Feature | Comments | Commit Messages | Lore |
|---------|----------|-----------------|------|
| **Volume** | Must be short | ~50 chars | Unlimited (5000+ words) |
| **Searchable** | Code only | Messages only | Full reasoning + alternatives |
| **Stability** | Often deleted | Can be amended | Immutable, hash-linked |
| **Context** | What the code does | What changed | Why it was written that way |

## Integration with AI Agents

Lore is designed to work seamlessly with AI coding assistants. Add to your agent's system instructions:

```
You have access to a tool called `lore`.

Before you edit a file, run `lore explain <file>` to understand the hidden context.

After you finish a task, run `lore record` to save your chain-of-thought so future agents understand your decisions.
```

### Benefits for Multi-Agent Workflows

When multiple AI agents work on the same codebase:

1. **No Lost Context**: Each agent's reasoning is preserved forever
2. **Avoid Repeated Mistakes**: Rejected alternatives prevent re-exploring dead ends
3. **Understand Intent**: Future agents know *why* code exists, not just *what* it does
4. **Searchable History**: Query reasoning across the entire project history

## Claude Code Integration

<img src="claude_logo.svg.png" alt="Claude" height="40">

Claude Code can be configured to automatically use Lore for every code change.

### Setup

1. **Initialize Lore in your project:**
   ```bash
   lore init --agent "claude-code"
   ```

2. **Create a `CLAUDE.md` file** in your project root:

   ```markdown
   # Lore Integration

   Before modifying any file, check for existing reasoning:
     lore explain <file-path>

   After completing any code change, record your reasoning:
     lore record -f <file> -m "Brief description" \
         --trace "Full reasoning..." \
         -r "Rejected alternative" \
         -T relevant-tag
   ```

3. **Commit the `CLAUDE.md` file** to your repository so all Claude Code sessions use it.

## Cursor Integration

<img src="cursor_logo.png" alt="Cursor" height="40">

Configure Cursor to use Lore for persistent reasoning context.

### Setup

1. **Initialize Lore in your project:**
   ```bash
   lore init --agent "cursor"
   ```

2. **Create a `.cursorrules` file** in your project root:

   ```markdown
   # Lore Integration for Cursor

   This project uses Lore to track reasoning behind code changes.

   ## Before Editing Files
   Before modifying any file, check if there's existing reasoning:
     lore explain <file-path>

   ## After Making Changes
   After completing code changes, record your reasoning:
     lore record -f <changed-file> \
         -m "Brief description of what you did" \
         --trace "Your full reasoning explaining:
     - Why you made these specific changes
     - What alternatives you considered
     - Any trade-offs or concerns" \
         -r "Alternative you rejected" \
         -T relevant-tag
   ```

### Alternative: Use Cursor Settings

Add to Cursor's "Rules for AI" in Settings > General > Rules for AI:

```
When working on code:
1. Before editing any file, run 'lore explain <file>' to check for existing context
2. After making changes, run 'lore record' with your reasoning
3. Include rejected alternatives with -r flag
4. Add relevant tags with -T flag
```

## Installation

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and install lore
cargo install --path .

# Or build without installing
cargo build --release
# Binary will be at target/release/lore
```

## Manual Usage

You can also use Lore directly from the command line:

```bash
# Initialize Lore in your project
lore init --agent "my-agent-id"

# After making code changes, record your reasoning
lore record -m "Refactoring auth to handle JWTs" \
    --trace "I initially tried using library X, but it conflicted with our dependencies..."

# Later, understand why code exists
lore explain src/auth_middleware.py

# Search through reasoning history
lore search "JWT"
lore search "pandas"  # Find all code avoiding pandas
```

## Commands Reference

### `lore init`

Initialize a new Lore repository.

```bash
lore init                          # Initialize in current directory
lore init --agent "my-agent-id"    # Set default agent ID
lore init --path /path/to/project  # Initialize in specific path
```

### `lore record`

Record reasoning for code changes.

```bash
# Auto-detect changed files from git
lore record -m "Brief intent" --trace "Full reasoning..."

# Specify files manually
lore record -f src/auth.py -f src/utils.py -m "Updated auth flow"

# Record with rejected alternatives
lore record -m "Chose manual JWT impl" \
    -r "Auth0 SDK" -r "Custom decorator approach"

# Record reasoning for specific lines
lore record -f src/auth.py --lines "10-45" -m "JWT validation logic"

# Read reasoning from a file or stdin
lore record -m "Refactoring" --trace-file ./reasoning.txt
lore record -m "Refactoring" --stdin < reasoning.txt

# Add tags for categorization
lore record -m "Performance fix" -T performance -T critical
```

### `lore explain`

Retrieve reasoning behind a file.

```bash
lore explain src/auth_middleware.py        # Show most recent reasoning
lore explain src/auth_middleware.py --all  # Show full history
lore explain src/auth.py --json            # Output as JSON
lore explain src/auth.py --limit 5         # Limit to 5 entries
```

### `lore search`

Search through reasoning history.

```bash
lore search "JWT"                       # Search all reasoning
lore search "pandas" --file utils       # Filter by file
lore search "refactor" --agent claude   # Filter by agent
lore search "performance" --limit 10    # Limit results
lore search "auth" --json               # Output as JSON
```

### `lore list`

List all recorded entries.

```bash
lore list                # Show all entries
lore list --limit 20     # Limit to 20 entries
lore list --json         # Output as JSON
```

### `lore status`

Show Lore status for the repository.

```bash
lore status  # Shows entry count, tracked files, changed files without reasoning
```

## Data Storage

Lore stores data in `.lore/` folder (intended to be committed to Git):

```
.lore/
├── config.json       # Repository configuration
├── index.json        # File → entry ID mappings
├── entries/          # Individual thought objects
│   ├── uuid1.json
│   ├── uuid2.json
│   └── ...
└── .gitignore        # Ignores temp files
```

Each entry is a JSON file:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "target_file": "src/auth_middleware.py",
  "line_range": [10, 45],
  "file_hash": "sha256:...",
  "commit_hash": "a1b2c3d4...",
  "agent_id": "claude-3-5-sonnet",
  "timestamp": "2024-02-14T10:00:00Z",
  "intent": "Refactoring auth to handle JWTs",
  "reasoning_trace": "I initially tried using library X...",
  "rejected_alternatives": [
    {"name": "Auth0 SDK", "reason": "Dependency conflicts"}
  ],
  "tags": ["auth", "security"]
}
```

## Author

Built by [Avraam Mavridis](https://www.avraam.dev/) &bull; [LinkedIn](https://www.linkedin.com/in/avrmav/)

## License

MIT
