#!/bin/bash
set -e

# 1. Compile mcpd
echo "Compiling mcpd..."
cargo build --target-dir /tmp/rust-target

# 2. Run mcpd in background
echo "Starting mcpd..."
/tmp/rust-target/debug/mcpd > mcpd.log 2>&1 &
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
    echo "mcpd socket not found after 5 seconds"
    cat mcpd.log
    kill $MCPD_PID
    exit 1
fi

# 3. Run orchestrator test
echo "Running orchestrator test..."
NODE_OPTIONS="--loader ts-node/esm --no-warnings" npx ts-node test_e2e.ts || { echo "Orchestrator test failed"; kill $MCPD_PID; exit 1; }

# 4. Cleanup
echo "Cleaning up..."
kill $MCPD_PID
echo "E2E verification successful!"
