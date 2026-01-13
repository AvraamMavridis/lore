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

## Quick Start

```bash
# Initialize Lore in your project
lore init --agent "claude-3-5-sonnet"

# After making code changes, record your reasoning
lore record -m "Refactoring auth to handle JWTs" \
    --trace "I initially tried using library X, but it conflicted with our dependencies..."

# Later, understand why code exists
lore explain src/auth_middleware.py

# Search through reasoning history
lore search "JWT"
lore search "pandas"  # Find all code avoiding pandas
```

## Commands

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

## Integration with AI Agents

Add to your agent's system instructions:

```
You have access to a tool called `lore`.

Before you edit a file, run `lore explain <file>` to understand the hidden context.

After you finish a task, run `lore record` to save your chain-of-thought so future agents understand your decisions.
```

## Claude Code Integration

Claude Code can be configured to automatically use Lore for every code change. Add a `CLAUDE.md` file to your project root with instructions for Claude Code to follow.

### Setup

1. **Initialize Lore in your project:**
   ```bash
   lore init --agent "claude-code"
   ```

2. **Create a `CLAUDE.md` file** in your project root:

   ```markdown
   # Lore Integration

   This project uses Lore to track reasoning behind code changes.

   ## Before Editing Files

   Before modifying any file, check if there's existing reasoning context:

   lore explain <file-path>
   

   This will show you:
   - Why the code was written this way
   - What alternatives were considered and rejected
   - Any warnings or notes from previous developers/agents

   ## After Making Changes

   After completing any code change, you MUST record your reasoning:

  
   lore record -f <changed-file> \
       -m "Brief description of what you did" \
       --trace "Your full reasoning and chain-of-thought explaining:
   - Why you made these specific changes
   - What alternatives you considered
   - Any trade-offs or concerns
   - Warnings for future developers" \
       -r "Alternative 1 you rejected" \
       -r "Alternative 2 you rejected" \
       -T relevant-tag
   

   ### Example

   If you refactored the authentication module:

   
   lore record -f src/auth.py \
       -m "Refactored JWT validation to handle refresh tokens" \
       --trace "The existing implementation only supported access tokens. I added refresh token support by:
   1. Adding a token_type field to distinguish token types
   2. Creating separate validation paths for each type
   3. Adding a refresh endpoint

   I considered using a separate RefreshToken class but decided against it to keep the codebase simpler. The current approach handles both types in the same Token class with a discriminator field.

   Warning: The refresh token expiry is hardcoded to 7 days. This should probably be configurable." \
       -r "Separate RefreshToken class" \
       -r "Third-party JWT library" \
       -T auth -T jwt -T refactoring
   

   ## Important

   - Always run `lore explain` before editing unfamiliar code
   - Always run `lore record` after completing changes
   - Include rejected alternatives to help future agents avoid dead ends
   - Add warnings about fragile or non-obvious code behavior
   ```

3. **Commit the `CLAUDE.md` file** to your repository so all Claude Code sessions use it.

> **Tip:** Copy the template from `examples/CLAUDE.md.template` in this repository for a ready-to-use configuration.

### Alternative: User-Level Configuration

For personal projects or to apply Lore globally, you can add instructions to your Claude Code settings:

1. Open Claude Code settings (usually `~/.claude/settings.json` or via the CLI)

2. Add custom instructions:
   ```json
   {
     "customInstructions": "When working on code:\n1. Before editing any file, run 'lore explain <file>' to check for existing context\n2. After making changes, run 'lore record' with your reasoning\n3. Include rejected alternatives with -r flag\n4. Add relevant tags with -T flag"
   }
   ```

### What Gets Recorded

When Claude Code follows these instructions, each change will capture:

| Field | Example |
|-------|---------|
| **Intent** | "Refactored JWT validation to handle refresh tokens" |
| **Reasoning Trace** | Full explanation of approach, trade-offs, and concerns |
| **Rejected Alternatives** | "Separate RefreshToken class", "Third-party JWT library" |
| **Tags** | `#auth`, `#jwt`, `#refactoring` |
| **File Hash** | Cryptographic link to exact file state |
| **Timestamp** | When the change was made |
| **Agent ID** | "claude-code" |

### Benefits for Multi-Agent Workflows

When multiple Claude Code sessions (or other AI agents) work on the same codebase:

1. **No Lost Context**: Each agent's reasoning is preserved forever
2. **Avoid Repeated Mistakes**: Rejected alternatives prevent re-exploring dead ends
3. **Understand Intent**: Future agents know *why* code exists, not just *what* it does
4. **Searchable History**: Query reasoning across the entire project history

```bash
# Find all code written to avoid a specific library
lore search "pandas"

# Find all authentication-related decisions
lore search "auth" --tag auth

# See full history of a frequently-modified file
lore explain src/core/engine.py --all
```

### Example Agent Workflow

```bash
# Agent reads existing reasoning before editing
$ lore explain src/auth_middleware.py
═══════════════════════════════════════════════════════════════
Lore for: src/auth_middleware.py
═══════════════════════════════════════════════════════════════

Agent: claude-3-5-sonnet │ 2024-02-14 10:00:00 UTC
Commit: a1b2c3d4

Intent:
Refactoring auth to handle JWTs

Reasoning:
  I initially tried using library X, but it conflicted with our
  dependencies. I switched to a manual implementation.

  Note: This logic is brittle if the token format changes.

Rejected Alternatives:
  ✗ Auth0 SDK
  ✗ Custom decorator approach

═══════════════════════════════════════════════════════════════

# Agent makes changes, then records reasoning
$ lore record -m "Extended JWT handling for refresh tokens" \
    --trace "Building on the previous implementation, I added..." \
    -r "Separate refresh token service"
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

## Why Lore?

| Feature | Comments | Commit Messages | Lore |
|---------|----------|-----------------|------|
| **Volume** | Must be short | ~50 chars | Unlimited (5000+ words) |
| **Searchable** | Code only | Messages only | Full reasoning + alternatives |
| **Stability** | Often deleted | Can be amended | Immutable, hash-linked |
| **Context** | What the code does | What changed | Why it was written that way |

## License

MIT
