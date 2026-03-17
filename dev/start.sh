#!/usr/bin/env bash
# Start the full local dev stack in one terminal.
# Ctrl-C stops everything cleanly.
set -e

# Clear ports
lsof -ti:3000,4840,4841 | xargs kill -9 2>/dev/null || true
sleep 0.3
mkdir -p jobs/input jobs/machine jobs/output

# Build both binaries
cargo build --bin mock-plat-trunk --bin opcua-howick 2>&1 | grep -E "^error|Compiling|Finished"

# Start mock plat-trunk
cargo run --bin mock-plat-trunk &
MOCK_PID=$!

# Start opcua-howick
RUST_LOG=opcua_howick=info cargo run --bin opcua-howick &
SERVER_PID=$!

echo ""
echo "Running:"
echo "  mock-plat-trunk  PID $MOCK_PID   →  http://localhost:3000"
echo "  opcua-howick     PID $SERVER_PID  →  opc.tcp://localhost:4840 | http://localhost:4841"
echo ""
echo "Test commands (new terminal):"
echo "  mise run dev:job     — drop a test CSV into jobs/input/"
echo "  mise run dev:status  — check machine state JSON"
echo "  mise run dev:browse  — OPC UA endpoint for UaExpert"
echo ""
echo "Ctrl-C to stop both"

cleanup() {
    echo ""
    echo "Stopping..."
    kill $MOCK_PID $SERVER_PID 2>/dev/null
    exit 0
}
trap cleanup INT TERM

wait
