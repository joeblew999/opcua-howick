#!/usr/bin/env bash
# Full dev stack — mirrors production hardware:
#
#   opcua-howick  reads opcua-howick.dev.toml        (same as opcua-howick.pi5.toml on Pi 5)
#   howick-agent  reads howick-agent.dev.toml  (same as howick-agent.pi-zero.toml on Pi Zero)
#
# Each binary has its own config file — no env var overrides needed.
# This is the same pattern used on hardware, just with localhost addresses.
#
# Dashboard: http://localhost:4841/dashboard
# Drop a job: mise run dev:job
# Check state: mise run dev:status

set -e

# Binaries already built by mise depends=[build] — just start them.
lsof -ti:4840,4841 | xargs kill -9 2>/dev/null || true
sleep 0.3
mkdir -p jobs/input jobs/machine jobs/output

RUST_LOG=opcua_howick=info cargo run --bin opcua-howick &
SERVER_PID=$!
sleep 2

RUST_LOG=howick_agent=info cargo run --bin howick-agent -- --config howick-agent.dev.toml &
AGENT_PID=$!

echo ""
echo "Pipeline running:"
echo "  opcua-howick  PID $SERVER_PID  →  opc.tcp://localhost:4840/"
echo "  opcua-howick  PID $SERVER_PID  →  http://localhost:4841/dashboard"
echo "  howick-agent  PID $AGENT_PID  →  subscribed via OPC UA (no polling)"
echo ""
echo "Open dashboard: http://localhost:4841/dashboard"
echo "Drop a job:     mise run dev:job"
echo ""
echo "Ctrl-C to stop"

cleanup() {
    echo ""
    echo "Stopping..."
    kill $SERVER_PID $AGENT_PID 2>/dev/null
    exit 0
}
trap cleanup INT TERM
wait
