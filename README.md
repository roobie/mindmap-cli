# mindmap-cli

A small CLI for inspecting and safely editing one-line MINDMAP files (default: ./MINDMAP.md).
One-node-per-line format: `[N] **Title** - body with [N] references`. IDs are stable numeric values.

## Quick Reference

| Task | Command |
|------|---------|
| **View a node** | `mindmap-cli show 10` |
| **Find nodes by type** | `mindmap-cli list --type AE` |
| **Search nodes** | `mindmap-cli search auth` or `mindmap-cli list --grep auth` |
| **Find incoming references** | `mindmap-cli refs 10` (← nodes referring to [10]) |
| **Find outgoing references** | `mindmap-cli links 10` (→ nodes that [10] refers to) |
| **Add a node** | `mindmap-cli add --type AE --title "Title" --body "Description [12]"` |
| **Edit a node** | `mindmap-cli edit 12` (opens $EDITOR) |
| **Update a node** | `mindmap-cli patch 12 --title "New Title"` |
| **Replace a node** | `mindmap-cli put 12 --line "[12] **AE: Title** - body [10]"` |
| **Delete a node** | `mindmap-cli delete 12 --force` |
| **Find orphans** | `mindmap-cli orphans` or `mindmap-cli orphans --with-descriptions` |
| **Validate file** | `mindmap-cli lint` or `mindmap-cli lint --fix` |
| **Batch operations** | `mindmap-cli batch --input commands.txt` |
| **Output as JSON** | Add `--output json` to any command |
| **Use stdin** | `mindmap-cli --file - show 10` (read-only) |

## Core usage examples:

```bash
# Print primer (e.g. when starting a new conversation with your coding agent)
mindmap-cli prime

# show a node
mindmap-cli show 10

# list nodes (filter by type or grep)
mindmap-cli list --type AE --grep auth

# similar to list --grep
mindmap-cli search auth

# add a node (auto picks next free ID)
mindmap-cli add --type AE --title "AuthService" --body "Handles auth [12]"

# (interactive) add (opens $EDITOR and tries to store the saved data as a new node)
mindmap-cli add

# (interactive) open node in $EDITOR for safe edit
mindmap-cli edit 12

# partial (scriptable) update: change title and/or body
mindmap-cli patch 12 --title "AuthSvc" --body "Updated body"

# full-line replace (must keep same id)
mindmap-cli put 12 --line "[12] **AE: AuthSvc** - Updated body [10]"

# delete a node (use --force to remove even if referenced)
mindmap-cli delete 12 --force

# BATCH non-interactive mutations:
mindmap-cli batch --input - <<EOF
  add --type AE --title "AuthService" --body "Handles auth [12]"
  put 12 --line "[12] **AE: AuthSvc** - Updated body [10]"
  patch 14 --title "TelemetrySvc" --body "Updated body ..."
  delete 22
EOF

# lint the file for syntax / ref issues
mindmap-cli lint

# list orphan nodes (no incoming and no outgoing references)
mindmap-cli orphans --with-descriptions

# Examples: piping JSON output and separating stderr (meta) from stdout (data)

# show node 12 as JSON and extract the node id
mindmap-cli --output json show 12 | jq '.node.id'

# show node 12 while capturing meta on stderr (Bash example)
mindmap-cli show 12 1>data.json 2>meta.log
```

For more details run `mindmap-cli --help` or `mindmap-cli <subcommand> --help` or see the [DESIGN.md](DESIGN.md) files in this repository.

## Understanding Refs vs Links

- **Refs** (← INCOMING): "Who references this node?" - Find all nodes that point TO the given node
- **Links** (→ OUTGOING): "What does this node reference?" - Find all nodes that the given node points TO

Example:
```
[1] **AE: Service** - handles auth [2] [3]
[2] **DB: UserDB** - stores users
[3] **AE: Cache** - caches data [2]

mindmap-cli refs 2
→ Shows [1] and [3] (nodes that refer to [2])

mindmap-cli links 1  
→ Shows [2] and [3] (nodes that [1] refers to)
```

## Output formats

By default the CLI prints human-readable output. Use the global flag `--output json` to emit structured JSON to stdout for scripting. Informational messages, confirmations, and warnings are written to stderr so stdout remains machine-actionable when you pipe the output.

Example (pipe JSON to jq):

```bash
# show node 12 as JSON and extract the node id
mindmap-cli --output json show 12 | jq '.node.id'

# list nodes as JSON
mindmap-cli --output json list --type AE | jq '.items'
```

