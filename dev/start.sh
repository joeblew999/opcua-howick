#!/usr/bin/env bash
# Full Option B pipeline — opcua-howick (queue mode) + howick-agent
#
# What runs:
#   opcua-howick  :4840 OPC UA | :4841 HTTP dashboard
#   howick-agent  polls opcua-howick for jobs, writes CSVs to jobs/machine/
#
# What to do:
#   open http://localhost:4841/dashboard
#   drag a CSV in — watch it flow to jobs/machine/
#
# Other terminal:
#   mise run dev:job      — drop T1.csv fixture into the pipeline
#   mise run dev:status   — check machine state JSON
#   mise run dev:sensor   — simulate coil weight reading

set -e

lsof -ti:4840,4841 | xargs kill -9 2>/dev/null || true
sleep 0.3
mkdir -p jobs/input jobs/machine jobs/output

cargo build --bin opcua-howick --bin howick-agent 2>&1 | grep -E "^error|Compiling|Finished"

DELIVERY_MODE=queue RUST_LOG=opcua_howick=info cargo run --bin opcua-howick &
SERVER_PID=$!
sleep 2

PLAT_TRUNK_URL=http://localhost:4841 RUST_LOG=howick_agent=info cargo run --bin howick-agent &
AGENT_PID=$!

echo ""
echo "Pipeline running:"
echo "  opcua-howick  PID $SERVER_PID  →  http://localhost:4841/dashboard"
echo "  howick-agent  PID $AGENT_PID  →  polling http://localhost:4841"
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
