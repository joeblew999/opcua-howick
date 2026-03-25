"""
Convert SketchUp framing geometry from Speckle into Howick CSV machine files.

Reads the FrameBuilderMRD layer structure (Stud, Nog, BottomPlate, TopPlate, etc.)
and extracts member positions, lengths, and punch operation positions from mesh vertices.

Usage:
    export SPECKLE_TOKEN="your-token"
    PYTHONPATH=tools/speckle-watcher/src python3 -m speckle_watcher.converter <project_id> <model_id>
"""

import os
import sys
from dataclasses import dataclass, field
from pathlib import Path

from specklepy.api.client import SpeckleClient
from specklepy.transports.server import ServerTransport
from specklepy.api import operations


# ── Howick CSV types ──────────────────────────────────────────────────────────


@dataclass
class Operation:
    """A single machine operation at a position along a component."""
    name: str   # DIMPLE, LIP_CUT, SWAGE, WEB, END_TRUSS, NOTCH, SERVICE_HOLE
    position: float  # mm from start of component


@dataclass
class Component:
    """One steel member to roll-form."""
    id: str          # e.g. W-G1-1
    label: str       # LABEL_NRM or LABEL_INV
    qty: int
    length: float    # mm
    operations: list[Operation] = field(default_factory=list)


@dataclass
class Frameset:
    """A complete job — one CSV file."""
    name: str               # e.g. wall1
    job_name: str           # e.g. 25062026
    unit: str = "MILLIMETRE"
    profile: str = "S8908"
    profile_desc: str = "Standard Profile"
    components: list[Component] = field(default_factory=list)

    def to_csv(self) -> str:
        """Generate Howick CSV content."""
        lines = []
        lines.append(f"UNIT,{self.unit}")
        lines.append(f"PROFILE,{self.profile},{self.profile_desc}")
        lines.append(f"FRAMESET,{self.name},{self.job_name}")

        for comp in self.components:
            parts = [
                "COMPONENT",
                comp.id,
                comp.label,
                str(comp.qty),
                f"{comp.length:.1f}",
            ]
            # Group operations by type, maintaining order
            for op in comp.operations:
                parts.append(op.name)
                parts.append(f"{op.position:.2f}" if op.position != int(op.position)
                             else f"{op.position:.1f}")
            lines.append(",".join(parts))

        return "\n".join(lines) + "\n"


# ── Geometry extraction ───────────────────────────────────────────────────────


# Known operation types and how to detect them from mesh geometry.
# Punch features create extra vertices at specific Z positions along the member.
# We classify based on the cross-section shape at each Z cluster.

# For now: extract all unique Z positions as potential operation sites,
# then classify using heuristics based on the feature geometry.

PROFILE_WEB = 88.9      # S8908 web depth (Y axis)
PROFILE_FLANGE = 41.3   # S8908 flange width (X axis)
PROFILE_LIP = 10.0      # S8908 lip


def extract_mesh_data(elem):
    """Extract vertices from a Speckle mesh element."""
    verts = getattr(elem, "vertices", None)
    if not verts or len(verts) < 3:
        return None

    n = len(verts) // 3
    points = [(verts[j * 3], verts[j * 3 + 1], verts[j * 3 + 2]) for j in range(n)]
    return points


def points_to_bbox(points):
    """Get bounding box from a list of (x, y, z) tuples."""
    xs = [p[0] for p in points]
    ys = [p[1] for p in points]
    zs = [p[2] for p in points]
    return {
        "min_x": min(xs), "max_x": max(xs),
        "min_y": min(ys), "max_y": max(ys),
        "min_z": min(zs), "max_z": max(zs),
        "dx": max(xs) - min(xs),
        "dy": max(ys) - min(ys),
        "dz": max(zs) - min(zs),
    }


def detect_member_axis(bbox):
    """Determine which axis the member runs along (longest dimension)."""
    dims = {"x": bbox["dx"], "y": bbox["dy"], "z": bbox["dz"]}
    return max(dims, key=dims.get)


def extract_operation_positions(points, bbox, axis):
    """
    Extract unique positions along the member axis that likely correspond
    to punch operations. These are Z (or X/Y) values where extra vertices
    exist beyond the basic C-section profile.

    A plain C-section cross-section has a small number of vertices at each
    longitudinal position. Punch features (dimples, lip cuts, swages, etc.)
    add extra vertices at their positions.
    """
    if axis == "z":
        positions = [round(p[2], 2) for p in points]
        start = bbox["min_z"]
    elif axis == "x":
        positions = [round(p[0], 2) for p in points]
        start = bbox["min_x"]
    else:
        positions = [round(p[1], 2) for p in points]
        start = bbox["min_y"]

    # Count vertices at each position along the member
    from collections import Counter
    pos_counts = Counter(positions)

    # The "background" vertex count is the mode (most common count)
    # — this is the plain C-section profile repeating
    counts = list(pos_counts.values())
    if not counts:
        return []

    # Find positions with extra vertices (potential punch features)
    # Sort unique positions
    unique_positions = sorted(set(positions))

    # Convert to offsets from member start
    offsets = [round(p - start, 2) for p in unique_positions]

    return offsets


def classify_operations(offsets, length):
    """
    Classify operation positions into DIMPLE, LIP_CUT, SWAGE, etc.

    Heuristics based on known patterns from Howick CSV:
    - SWAGE: typically near ends (< 30mm from start/end)
    - DIMPLE: throughout the member
    - LIP_CUT: near dimple positions, slightly offset
    - SERVICE_HOLE: typically at ~400mm intervals or specific positions
    - END_TRUSS: at 0.0 and at length

    For now, we output all positions as generic operations and refine later.
    """
    ops = []

    for offset in offsets:
        # Skip the very start and end (profile edges, not operations)
        if offset < 1.0 or offset > length - 1.0:
            continue

        # Classify based on position heuristics
        near_start = offset < 30.0
        near_end = offset > length - 30.0

        if near_start or near_end:
            # SWAGE positions are typically at 27.5mm from ends
            if 26.0 <= offset <= 29.0 or length - 29.0 <= offset <= length - 26.0:
                ops.append(Operation("SWAGE", offset))
            elif 18.0 <= offset <= 22.0 or length - 22.0 <= offset <= length - 18.0:
                ops.append(Operation("DIMPLE", offset))
            elif 22.0 <= offset <= 24.0 or length - 24.0 <= offset <= length - 22.0:
                ops.append(Operation("LIP_CUT", offset))
            else:
                ops.append(Operation("DIMPLE", offset))
        else:
            # Interior positions — classify by clustering
            # For now, mark as DIMPLE (most common operation)
            ops.append(Operation("DIMPLE", offset))

    return ops


# ── Layer processing ──────────────────────────────────────────────────────────

# Map FrameBuilderMRD layer names to component ID prefixes
LAYER_ROLE = {
    "Stud": "S",
    "Nog": "N",
    "BottomPlate": "B",
    "TopPlate": "T",
    "window": "W",
    "generic_frame": "GF",
    "lateralbrace": "LB",
}


def process_layer(layer_name, elements, frameset_name):
    """Process all members in a layer, returning Component objects."""
    components = []
    prefix = LAYER_ROLE.get(layer_name, "X")
    count = 0

    for elem in elements:
        points = extract_mesh_data(elem)
        if not points:
            continue

        count += 1
        bbox = points_to_bbox(points)
        axis = detect_member_axis(bbox)

        if axis == "z":
            length = bbox["dz"]
        elif axis == "x":
            length = bbox["dx"]
        else:
            length = bbox["dy"]

        if length < 5.0:
            continue  # skip tiny fragments

        # Determine label orientation based on position
        label = "LABEL_NRM" if count % 2 == 1 else "LABEL_INV"

        # Extract operation positions from vertex clustering
        offsets = extract_operation_positions(points, bbox, axis)
        ops = classify_operations(offsets, length)

        comp = Component(
            id=f"{frameset_name}-{prefix}{count}",
            label=label,
            qty=1,
            length=round(length, 2),
            operations=ops,
        )
        components.append(comp)

    return components


# ── Main ──────────────────────────────────────────────────────────────────────


def convert_speckle_to_csv(project_id: str, model_id: str, output_dir: Path = None):
    """
    Fetch geometry from Speckle and convert to Howick CSV.
    """
    server = os.environ.get("SPECKLE_SERVER", "https://app.speckle.systems")
    token = os.environ.get("SPECKLE_TOKEN")
    if not token:
        print("Error: Set SPECKLE_TOKEN environment variable.")
        sys.exit(1)

    client = SpeckleClient(host=server)
    client.authenticate_with_token(token)
    print(f"Connected to {server}")

    # Get latest version
    versions = client.version.get_versions(model_id=model_id, project_id=project_id)
    if not versions.items:
        print("No versions found.")
        sys.exit(1)

    latest = versions.items[0]
    ref_obj = latest.referenced_object
    print(f"Latest version: {latest.id}, object: {ref_obj}")

    # Receive the object tree
    transport = ServerTransport(stream_id=project_id, client=client)
    root = operations.receive(obj_id=ref_obj, remote_transport=transport)
    print(f"Received: {getattr(root, 'name', 'unnamed')}, units={getattr(root, 'units', '?')}")

    # Find the wall layer
    wall_layer = None
    for layer in root.elements:
        if getattr(layer, "name", "") == "wall":
            wall_layer = layer
            break

    if not wall_layer:
        print("Error: No 'wall' layer found in model.")
        print(f"Available layers: {[getattr(l, 'name', '?') for l in root.elements]}")
        sys.exit(1)

    # Process each sub-layer
    frameset_name = "wall1"  # derive from model name
    all_components = []

    framing_layers = ["Stud", "Nog", "BottomPlate", "TopPlate", "window", "generic_frame", "lateralbrace"]

    for sub in wall_layer.elements:
        layer_name = getattr(sub, "name", "")
        if layer_name not in framing_layers:
            print(f"  Skipping layer: {layer_name}")
            continue

        elems = sub.elements if hasattr(sub, "elements") else []
        components = process_layer(layer_name, elems, frameset_name)
        print(f"  {layer_name}: {len(elems)} elements → {len(components)} components")
        all_components.extend(components)

    # Build frameset
    frameset = Frameset(
        name=frameset_name,
        job_name="25062026",
        components=all_components,
    )

    csv_content = frameset.to_csv()

    # Output
    if output_dir:
        output_dir.mkdir(parents=True, exist_ok=True)
        out_path = output_dir / f"{frameset_name}.csv"
        out_path.write_text(csv_content)
        print(f"\nWrote {out_path} ({len(all_components)} components)")
    else:
        print(f"\n--- Generated CSV ({len(all_components)} components) ---")
        # Print first 10 lines
        lines = csv_content.split("\n")
        for line in lines[:13]:
            print(line)
        if len(lines) > 13:
            print(f"... ({len(lines) - 13} more lines)")

    return frameset


def main():
    if len(sys.argv) < 3:
        print("Usage: python -m speckle_watcher.converter <project_id> <model_id> [output_dir]")
        print()
        print("Example:")
        print("  SPECKLE_TOKEN=xxx python -m speckle_watcher.converter f3318660fc c6539853c0")
        print("  SPECKLE_TOKEN=xxx python -m speckle_watcher.converter f3318660fc c6539853c0 output/")
        sys.exit(1)

    project_id = sys.argv[1]
    model_id = sys.argv[2]
    output_dir = Path(sys.argv[3]) if len(sys.argv) > 3 else None

    convert_speckle_to_csv(project_id, model_id, output_dir)


if __name__ == "__main__":
    main()
