//! # opcua-howick
//!
//! OPC UA edge agent for Howick FRAMA roll-forming machines.
//!
//! Runs on a small compute module (Raspberry Pi, NUC, Mac Mini) next to the
//! Howick machine on the factory LAN. Bridges plat-trunk CAD output to the
//! physical machine via OPC UA + CSV file drop.
//!
//! See docs/architecture.md for full design.

fn main() {
    println!("opcua-howick: OPC UA edge agent for Howick FRAMA machines");
    println!("Status: skeleton — see docs/architecture.md");
}
