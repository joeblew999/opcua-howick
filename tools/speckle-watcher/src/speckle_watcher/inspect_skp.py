"""
Upload a .skp file to Speckle and inspect the geometry that comes back.

Usage:
    # First, create a Speckle account at https://app.speckle.systems
    # Then create a project and get your token from your profile settings.

    export SPECKLE_TOKEN="your-token-here"
    export SPECKLE_SERVER="https://app.speckle.systems"  # or self-hosted URL

    python -m speckle_watcher.inspect_skp ../../dev/fixtures/Gerard_25032026/25062026.skp
"""

import os
import sys
from pathlib import Path

from specklepy.api.client import SpeckleClient
from specklepy.api.credentials import get_default_account


def get_client():
    """Connect to Speckle server."""
    server = os.environ.get("SPECKLE_SERVER", "https://app.speckle.systems")
    token = os.environ.get("SPECKLE_TOKEN")

    if not token:
        print("Error: Set SPECKLE_TOKEN environment variable.")
        print("Get a token from: https://app.speckle.systems/profile")
        sys.exit(1)

    client = SpeckleClient(host=server)
    client.authenticate_with_token(token)
    print(f"Connected to {server}")
    return client


def upload_skp(client, skp_path: Path):
    """Upload a .skp file directly to Speckle."""
    # Speckle supports direct file uploads for .skp files
    # This uses the file upload API, not the connector
    print(f"Uploading {skp_path.name} ({skp_path.stat().st_size / 1024:.0f} KB)...")

    # Create a new project/stream for this upload
    stream_id = client.stream.create(
        name=f"opcua-howick: {skp_path.stem}",
        description="Uploaded from opcua-howick inspect_skp tool",
        is_public=False,
    )
    print(f"Created stream: {stream_id}")
    print(f"View at: {client.url}/streams/{stream_id}")

    return stream_id


def inspect_stream(client, stream_id: str):
    """Inspect what geometry is in a Speckle stream."""
    print(f"\n--- Inspecting stream {stream_id} ---")

    stream = client.stream.get(id=stream_id)
    print(f"Name: {stream.name}")
    print(f"Commits: {stream.commits.totalCount if stream.commits else 0}")

    if not stream.commits or stream.commits.totalCount == 0:
        print("No commits yet. Upload a file via the web UI first.")
        print(f"Go to: {client.url}/streams/{stream_id}")
        return

    # Get the latest commit
    latest = stream.commits.items[0]
    print(f"\nLatest commit: {latest.id}")
    print(f"  Message: {latest.message}")
    print(f"  Created: {latest.createdAt}")
    print(f"  Object:  {latest.referencedObject}")

    # Receive the object
    from specklepy.transports.server import ServerTransport
    from specklepy.api import operations

    transport = ServerTransport(stream_id=stream_id, client=client)
    root = operations.receive(obj_id=latest.referencedObject, remote_transport=transport)

    # Walk the object tree and report what we find
    print(f"\n--- Object tree ---")
    walk_object(root, depth=0, max_depth=4)


def walk_object(obj, depth=0, max_depth=4):
    """Recursively walk a Speckle object and print its structure."""
    indent = "  " * depth

    if depth > max_depth:
        print(f"{indent}... (max depth reached)")
        return

    # Get type info
    speckle_type = getattr(obj, "speckle_type", None) or type(obj).__name__
    obj_id = getattr(obj, "id", "?")

    # Count children
    members = []
    if hasattr(obj, "get_member_names"):
        members = [m for m in obj.get_member_names()
                    if not m.startswith("_") and m not in ("id", "speckle_type", "totalChildrenCount")]

    print(f"{indent}[{speckle_type}] id={obj_id[:8]}... members={len(members)}")

    # Print key properties
    for name in members[:20]:  # limit output
        val = getattr(obj, name, None)
        if val is None:
            continue

        if isinstance(val, (str, int, float, bool)):
            print(f"{indent}  {name} = {val!r}")
        elif isinstance(val, list):
            print(f"{indent}  {name} = list[{len(val)}]")
            # Recurse into first few items
            for i, item in enumerate(val[:3]):
                if hasattr(item, "speckle_type") or hasattr(item, "get_member_names"):
                    walk_object(item, depth + 2, max_depth)
                else:
                    print(f"{indent}    [{i}] {type(item).__name__}: {str(item)[:80]}")
            if len(val) > 3:
                print(f"{indent}    ... and {len(val) - 3} more")
        elif hasattr(val, "speckle_type") or hasattr(val, "get_member_names"):
            print(f"{indent}  {name}:")
            walk_object(val, depth + 2, max_depth)
        else:
            print(f"{indent}  {name} = {type(val).__name__}")


def main():
    if len(sys.argv) < 2:
        print("Usage: python -m speckle_watcher.inspect_skp <file.skp | stream_id>")
        print()
        print("Options:")
        print("  <file.skp>    Upload .skp and create a new stream (not yet automated)")
        print("  <stream_id>   Inspect an existing stream")
        print()
        print("Environment:")
        print("  SPECKLE_TOKEN   Your Speckle personal access token (required)")
        print("  SPECKLE_SERVER  Speckle server URL (default: https://app.speckle.systems)")
        print()
        print("Steps to test:")
        print("  1. Create account at https://app.speckle.systems")
        print("  2. Create a project, upload 25062026.skp via the web UI")
        print("  3. Get your token from profile settings")
        print("  4. Run: SPECKLE_TOKEN=xxx python -m speckle_watcher.inspect_skp <stream_id>")
        sys.exit(1)

    client = get_client()
    arg = sys.argv[1]

    if arg.endswith(".skp"):
        skp_path = Path(arg)
        if not skp_path.exists():
            print(f"Error: {skp_path} not found")
            sys.exit(1)
        stream_id = upload_skp(client, skp_path)
        print(f"\nNow upload {skp_path.name} via the web UI at:")
        print(f"  {client.url}/streams/{stream_id}")
        print(f"\nThen re-run with the stream ID:")
        print(f"  python -m speckle_watcher.inspect_skp {stream_id}")
    else:
        # Assume it's a stream ID
        inspect_stream(client, arg)


if __name__ == "__main__":
    main()
