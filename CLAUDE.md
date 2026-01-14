# Lore Integration

This project uses [Lore](https://github.com/your-org/lore) to track reasoning behind code changes. Lore creates a "high-context" repository where you can understand not just what the code does, but *why* it was written that way.

## Before Editing Files

Before modifying any file, check if there's existing reasoning context:

```bash
lore explain <file-path>
```

This will show you:
- Why the code was written this way
- What alternatives were considered and rejected
- Any warnings or notes from previous developers/agents

**Example:**
```bash
lore explain src/auth/middleware.py
```

If no reasoning exists, you'll see a message indicating this is new or undocumented code.

## After Making Changes

After completing any code change, you MUST record your reasoning using `lore record`:

```bash
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
```

### Parameters

| Flag | Description | Required |
|------|-------------|----------|
| `-f <file>` | File(s) to record reasoning for (auto-detects from git if omitted) | No |
| `-m "message"` | Brief intent/purpose (1-2 sentences) | Yes |
| `--trace "..."` | Full reasoning trace (can be extensive) | Yes |
| `-r "alternative"` | Rejected alternative (can use multiple times) | No |
| `-T tag` | Tag for categorization (can use multiple times) | No |
| `--lines "10-45"` | Specific line range if reasoning applies to subset | No |

### Example

If you refactored the authentication module:

```bash
lore record -f src/auth.py \
    -m "Refactored JWT validation to handle refresh tokens" \
    --trace "The existing implementation only supported access tokens. I added refresh token support by:

1. Adding a token_type field to distinguish token types
2. Creating separate validation paths for each type
3. Adding a refresh endpoint at /auth/refresh

I considered using a separate RefreshToken class but decided against it to keep the codebase simpler. The current approach handles both types in the same Token class with a discriminator field.

Trade-offs:
- Pro: Simpler code, single validation path with branching
- Con: Token class is now doing more than one thing

Warning: The refresh token expiry is hardcoded to 7 days in config.py:42. This should probably be made configurable via environment variable." \
    -r "Separate RefreshToken class" \
    -r "python-jose library (dependency conflicts)" \
    -r "Storing refresh tokens in Redis" \
    -T auth -T jwt -T refactoring
```

## Searching Past Reasoning

You can search through all recorded reasoning:

```bash
# Search for any mention of a term
lore search "JWT"

# Search within specific files
lore search "validation" --file auth

# Search by agent
lore search "refactor" --agent claude
```

## Checking Status

To see what files have reasoning recorded and which don't:

```bash
lore status
```

## Important Guidelines

1. **Always run `lore explain` before editing unfamiliar code** - This prevents you from undoing intentional decisions or re-exploring rejected approaches

2. **Always run `lore record` after completing changes** - Even small changes benefit from context

3. **Include rejected alternatives** - This is crucial! It helps future agents avoid dead ends you already explored

4. **Add warnings about fragile code** - If something looks wrong but is intentional, explain why

5. **Use descriptive tags** - Tags make reasoning searchable (e.g., `-T security`, `-T performance`, `-T bugfix`)

6. **Be verbose in reasoning traces** - Unlike comments, Lore entries can be as long as needed. More context is always better.
