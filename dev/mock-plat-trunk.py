#!/usr/bin/env python3
"""
Mock plat-trunk backend for local development.

Simulates the two endpoints howick-agent polls:
  GET  /api/jobs/howick/pending          — returns one test job, then empty
  POST /api/jobs/howick/{job_id}/complete — acknowledges completion

Usage:
  python3 dev/mock-plat-trunk.py
  # or: mise run dev:mock

Once running, start howick-agent in another terminal:
  mise run dev:agent

The agent will poll, receive the test job, write it to ./jobs/machine/,
and mark it complete. Check ./jobs/machine/TEST-W1.csv to confirm.
"""

import http.server
import json
import re
import sys

TEST_JOB = {
    "job_id": "dev-001",
    "frameset_name": "TEST-W1",
    "csv": (
        "UNIT,MILLIMETRE\n"
        "PROFILE,S8908,Standard Profile\n"
        "FRAMESET,TEST-W1\n"
        "COMPONENT,TEST-W1-1,LABEL_NRM,1,2400.0,DIMPLE,20.65,DIMPLE,70.65\n"
        "COMPONENT,TEST-W1-2,LABEL_NRM,1,1800.0,DIMPLE,20.65,DIMPLE,70.65\n"
    ),
}

job_served = False


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        global job_served
        if self.path == "/api/jobs/howick/pending":
            jobs = [] if job_served else [TEST_JOB]
            body = json.dumps({"jobs": jobs}).encode()
            self._respond(200, body)
            if jobs:
                print(f"  → served job: {TEST_JOB['job_id']} ({TEST_JOB['frameset_name']})")
        else:
            self._respond(404, b'{"error":"not found"}')

    def do_POST(self):
        global job_served
        if re.search(r"/api/jobs/howick/[^/]+/complete", self.path):
            length = int(self.headers.get("Content-Length", 0))
            self.rfile.read(length)
            job_served = True
            self._respond(200, b'{"ok":true}')
            print(f"  → job marked complete — queue now empty")
        else:
            self._respond(404, b'{"error":"not found"}')

    def _respond(self, code, body):
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, fmt, *args):
        print(f"[mock] {fmt % args}")


PORT = 3000
print(f"Mock plat-trunk running on http://localhost:{PORT}")
print(f"  Serving job: {TEST_JOB['job_id']} — {TEST_JOB['frameset_name']}")
print(f"  Start agent: mise run dev:agent")
print()

try:
    http.server.HTTPServer(("", PORT), Handler).serve_forever()
except KeyboardInterrupt:
    print("\nStopped.")
    sys.exit(0)
