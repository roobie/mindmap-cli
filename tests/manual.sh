#!/bin/bash

cat > /tmp/test_real_race.sh << 'EOF'
#!/bin/bash

# Create initial file
echo "[1] **AE: Original** - content" > /tmp/real_race.md

# Function to simulate slow batch (sleeps between stages)
slow_batch() {
    cargo run --quiet -- batch --file /tmp/real_race.md --input <(
        # Simulate a multi-line batch that takes time
        echo 'add --type WF --title "Step 1" --body "First op"'
        sleep 1  # Sleep during batch processing
        echo 'add --type WF --title "Step 2" --body "Second op"'
    ) 2>&1
}

# Start slow batch in background
slow_batch &
BATCH_PID=$!

# Wait a bit for batch to start, then modify file
sleep 0.5
echo "[2] **AE: Concurrent** - added by another process" >> /tmp/real_race.md
echo "Modified file at $(date)"

# Wait for batch to finish
wait $BATCH_PID
STATUS=$?

echo "Batch exit status: $STATUS"
echo ""
echo "Final file:"
cat /tmp/real_race.md
EOF

chmod +x /tmp/test_real_race.sh
bash /tmp/test_real_race.sh

# ==============================================================================

cat > /tmp/final_batch_test.sh << 'EOF'
#!/bin/bash
set -e

echo "=== Comprehensive Batch Command Test ==="
echo ""

# Create a test mindmap
cat > /tmp/batch_final.md << 'INIT'
Header: Test Mindmap

[1] **AE: Authentication** - handles user login
[2] **AE: AE: Duplicate** - has duplicate prefix (should be fixed)
[3] **WF: Workflow** - manages tasks
INIT

echo "Initial file:"
cat /tmp/batch_final.md
echo ""

# Create a batch operations file
cat > /tmp/batch_final_ops.txt << 'OPS'
# Add new nodes
add --type DR --title "Architecture decision" --body "Use microservices [1]"
add --type META --title "Status" --body "In progress"

# Patch existing nodes
patch 2 --title "Authorization module"
patch 3 --body "manages tasks and workflows [1]"

# Deprecate old node
deprecate 3 --to 1
OPS

echo "Batch operations (lines format):"
cat /tmp/batch_final_ops.txt
echo ""

echo "=== Test 1: Dry-run preview ==="
cargo run --quiet -- batch --file /tmp/batch_final.md --input /tmp/batch_final_ops.txt --dry-run 2>&1 | tail -20
echo ""

echo "=== Test 2: Apply batch with auto-fix ==="
cargo run --quiet -- batch --file /tmp/batch_final.md --input /tmp/batch_final_ops.txt --fix 2>&1
echo ""

echo "Final file:"
cat /tmp/batch_final.md
echo ""

echo "=== Test 3: Run lint to verify integrity ==="
cargo run --quiet -- lint --file /tmp/batch_final.md 2>&1
echo ""

echo "=== Test 4: List all nodes to confirm ==="
cargo run --quiet -- list --file /tmp/batch_final.md 2>&1
echo ""

echo "✓ All tests passed!"
EOF

chmod +x /tmp/final_batch_test.sh
bash /tmp/final_batch_test.sh
