#!/bin/bash
set -e

# Cleanup any previous state
rm -f tool_registry.json transactions.log transactions.log.1 transactions.log.2 mcpd.log

# 1. Compile
echo "Compiling workspace..."
cargo build

# 2. Run mcpd in background
echo "Starting mcpd..."
./target/debug/mcpd > mcpd.log 2>&1 &
MCPD_PID=$!

# Wait for socket to be ready
echo "Waiting for mcpd socket..."
for i in {1..10}; do
    if [ -S /tmp/mcpd.sock ]; then
        break
    fi
    sleep 0.5
done

if [ ! -S /tmp/mcpd.sock ]; then
    echo "ERROR: mcpd socket not found after 5 seconds"
    cat mcpd.log
    kill $MCPD_PID
    exit 1
fi

# 3. Run orchestrator test with simulated user confirmation (y)
echo "Running orchestrator test..."
# Note: camera.capture requires confirmation because of 'privacy_sensitive' capability
# We use 'yes' to provide 'y' continuously for all confirmation prompts
yes y | NODE_OPTIONS="--loader ts-node/esm --no-warnings" npx ts-node test_e2e.ts > test_output.log 2>&1 || {
    echo "ERROR: Orchestrator test failed"
    cat test_output.log
    kill $MCPD_PID
    exit 1
}

# 4. Verify results
echo "Verifying results..."

# Verify response status in output
if ! grep -q "status\":\"success" test_output.log; then
    echo "ERROR: Success status not found in test output"
    cat test_output.log
    kill $MCPD_PID
    exit 1
fi

# Verify transaction log exists and has content
if [ ! -f transactions.log ]; then
    echo "ERROR: transactions.log not created"
    kill $MCPD_PID
    exit 1
fi

# Verify success message in orchestrator output
if ! grep -q "E2E Test completed successfully" test_output.log; then
    echo "ERROR: E2E Test did not complete successfully"
    cat test_output.log
    kill $MCPD_PID
    exit 1
fi

# 5. Cleanup
echo "Cleaning up..."
kill $MCPD_PID
rm -f /tmp/mcpd.sock

echo "E2E verification successful!"
