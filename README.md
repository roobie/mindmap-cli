# mindmap-cli

A small CLI for inspecting and safely editing one-line MINDMAP files (default: ./MINDMAP.md).
One-node-per-line format: `[N] **Title** - description with [N] references`. IDs are stable numeric values.

Core usage examples:

```bash
# show a node
mindmap-cli show 10

# list nodes (filter by type or grep)
mindmap-cli list --type AE --grep auth

# add a node (auto picks next free ID)
mindmap-cli add --type AE --title "AuthService" --desc "Handles auth [12]"

# open node in $EDITOR for safe edit
mindmap-cli edit 12

# partial (scriptable) update: change title and/or description
mindmap-cli patch 12 --title "AuthSvc" --desc "Updated desc"

# full-line replace (must keep same id)
mindmap-cli put 12 --line "[12] **AE: AuthSvc** - Updated desc [10]"

# delete a node (use --force to remove even if referenced)
mindmap-cli delete 12 --force

# lint the file for syntax / ref issues
mindmap-cli lint
```

For more details run `mindmap-cli --help` or see the DESIGN.md and CHECKLIST.md files in this repository.

Output formats

By default the CLI prints human-readable output. Use the global flag `--output json` to emit structured JSON to stdout for scripting. Informational messages, confirmations, and warnings are written to stderr so stdout remains machine-actionable when you pipe the output.

Example (pipe JSON to jq):

```bash
# show node 12 as JSON and extract the node id
mindmap-cli --output json show 12 | jq '.node.id'

# list nodes as JSON
mindmap-cli --output json list --type AE | jq '.items'
```

