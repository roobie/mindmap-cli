# mindmap-cli

A small CLI for inspecting and safely editing one-line MINDMAP files (default: ./MINDMAP.md).
One-node-per-line format: `[N] **Title** - description with [N] references`. IDs are stable numeric values.

Core usage examples:

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
mindmap-cli add --type AE --title "AuthService" --desc "Handles auth [12]"

# (interactive) add (opens $EDITOR and tries to store the saved data as a new node)
mindmap-cli add

# (interactive) open node in $EDITOR for safe edit
mindmap-cli edit 12

# partial (scriptable) update: change title and/or description
mindmap-cli patch 12 --title "AuthSvc" --desc "Updated desc"

# full-line replace (must keep same id)
mindmap-cli put 12 --line "[12] **AE: AuthSvc** - Updated desc [10]"

# delete a node (use --force to remove even if referenced)
mindmap-cli delete 12 --force

# BATCH non-interactive mutations:
mindmap-cli batch --input - <<EOF
  add --type AE --title "AuthService" --desc "Handles auth [12]"
  put 12 --line "[12] **AE: AuthSvc** - Updated desc [10]"
  patch 14 --title "TelemetrySvc" --desc "Updated desc ..."
  delete 22
EOF

# lint the file for syntax / ref issues
mindmap-cli lint

# list orphan nodes (no incoming and no outgoing references)
mindmap-cli orphans

# Examples: piping JSON output and separating stderr (meta) from stdout (data)

# show node 12 as JSON and extract the node id
mindmap-cli --output json show 12 | jq '.node.id'

# show node 12 while capturing meta on stderr (Bash example)
mindmap-cli show 12 1>data.json 2>meta.log
```

For more details run `mindmap-cli --help` or `mindmap-cli <subcommand> --help` or see the [DESIGN.md](DESIGN.md) files in this repository.

## Output formats

By default the CLI prints human-readable output. Use the global flag `--output json` to emit structured JSON to stdout for scripting. Informational messages, confirmations, and warnings are written to stderr so stdout remains machine-actionable when you pipe the output.

Example (pipe JSON to jq):

```bash
# show node 12 as JSON and extract the node id
mindmap-cli --output json show 12 | jq '.node.id'

# list nodes as JSON
mindmap-cli --output json list --type AE | jq '.items'
```

