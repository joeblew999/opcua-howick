#!/usr/bin/env bash
# Full dev stack — opcua-howick (server) + howick-agent (OPC UA client)
#
# What runs:
#   opcua-howick  :4840 OPC UA server | :4841 HTTP dashboard
#   howick-agent  subscribes to :4840 via OPC UA — server pushes jobs instantly
#
# What to do:
#   open http://localhost:4841/dashboard
#   mise run dev:job    — drop T1.csv fixture into the pipeline
#   mise run dev:status — check machine state JSON

set -e

lsof -ti:4840,4841 | xargs kill -9 2>/dev/null || true
sleep 0.3
mkdir -p jobs/input jobs/machine jobs/output

cargo build --bin opcua-howick --bin howick-agent 2>&1 | grep -E "^error|Compiling|Finished"

DELIVERY_MODE=queue RUST_LOG=opcua_howick=info cargo run --bin opcua-howick &
SERVER_PID=$!
sleep 2

# OPC UA M2M — agent subscribes to Pi 5 server, no polling
PLAT_TRUNK_URL=opc.tcp://127.0.0.1:4840/ RUST_LOG=howick_agent=info cargo run --bin howick-agent &
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
