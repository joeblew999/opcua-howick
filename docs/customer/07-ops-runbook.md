# Document 7 of 7 — Operations Runbook
## Howick FRAMA — Full Lifecycle

**Prepared by:** Gerard Webb, ubuntu Software
**Date:** March 2026
**Audience:** Gerard — internal operations reference

---

## How updates reach the Pis

```
Code push (MacBook)
    ↓ git push → GitHub Actions CI
    ↓ builds all binaries → GitHub Release artifacts
    ↓
Pi Zero 2W + Pi 5 (factory, Si Racha)
    ↓ systemd timer checks GitHub Releases every hour
    ↓ downloads new binary if version changed
    ↓ restarts service automatically
```

No manual steps after initial provisioning.

---

## First-time provisioning

### Prerequisites (MacBook)
```bash
cargo install cross          # cross-compilation
mise install                 # installs pandoc + typst
```

### Pi Zero 2W — two steps (USB gadget setup requires reboot)

**Step 1** — on same WiFi as the Pi (just after flash):
```bash
export ZERO_HOST=pi@howick-pi-zero.local
mise run setup:first-boot:pi-zero
# Pi reboots at end — wait ~30 seconds
```

**Step 2** — via Tailscale from anywhere:
```bash
export ZERO_HOST=pi@100.x.x.x   # Tailscale IP from step 1
mise run setup:post-reboot:pi-zero
# deploys binary + howick-agent.pi-zero.toml → ~/config.toml (first time only)
```

Full Pi Zero USB gadget detail in Document 6 (Pi Zero Setup).

### Pi 5 — single step

```bash
export PI5_HOST=pi@howick-pi5.local
mise run setup:first-boot:pi5
# deploys binary + opcua-howick.pi5.toml → ~/config.toml (first time only)
```

---

## Day-to-day

### Deploy a new version manually (urgent fix)
```bash
ZERO_HOST=pi@100.x.x.x mise run deploy:pi-zero
PI5_HOST=pi@100.x.x.x  mise run deploy:pi5
# Config is NOT overwritten — only the binary updates
# To force-reset config: scp howick-agent.pi-zero.toml pi@100.x.x.x:~/config.toml
```

### Trigger update check immediately (don't wait an hour)
```bash
ZERO_HOST=pi@100.x.x.x mise run update:check:pi-zero
PI5_HOST=pi@100.x.x.x  mise run update:check:pi5
```

### SSH
```bash
mise run ssh:pi-zero    # ZERO_HOST must be set
mise run ssh:pi5        # PI5_HOST must be set
```

### Logs
```bash
mise run logs:pi-zero   # stream howick-agent logs
mise run logs:pi5       # stream opcua-howick logs
```

### Status
```bash
mise run status:pi-zero  # systemd service status
mise run status:pi5      # systemd + HTTP API
```

### Check auto-update timer
```bash
ssh $ZERO_HOST 'systemctl list-timers howick-agent-update.timer'
ssh $PI5_HOST  'systemctl list-timers opcua-howick-update.timer'
```

### Check installed version
```bash
ssh $ZERO_HOST 'cat ~/.howick-agent-version'
ssh $PI5_HOST  'cat ~/.opcua-howick-version'
```

---

## Release process

1. Merge to `master` — CI: check + fmt + test
2. Create GitHub Release (tag e.g. `v0.2.0`)
3. CI builds all binaries and attaches to release
4. Both Pis pick up the new version within 1 hour automatically

**Force immediate update:**
```bash
mise run update:check:pi-zero
mise run update:check:pi5
```

---

## Auto-update internals

| Timer | Fires | Script |
|-------|-------|--------|
| `howick-agent-update.timer` | 5min after boot, then every hour | `/usr/local/bin/howick-agent-update.sh` |
| `opcua-howick-update.timer` | 5min after boot, then every hour | `/usr/local/bin/opcua-howick-update.sh` |

Each script:
1. Calls GitHub API for latest release tag
2. Compares with version file on Pi (`~/.howick-agent-version` or `~/.opcua-howick-version`)
3. If newer: downloads binary, writes version file, restarts service
4. If same: exits silently

Logs: `journalctl -u howick-agent-update` / `journalctl -u opcua-howick-update`

---

## Secrets (Doppler)

All secrets live in Doppler — never written to disk on the Pi.

| Project | Config | Used by |
|---------|--------|---------|
| `opcua-howick` | `pi-zero` | howick-agent on Pi Zero 2W |
| `opcua-howick` | `pi5` | opcua-howick on Pi 5 |

```bash
mise run doppler:secrets              # list secrets locally
mise run doppler:setup:pi-zero        # re-auth Pi Zero
mise run doppler:setup:pi5            # re-auth Pi 5
```

---

## Monitoring (Pi 5)

Netdata: `http://<pi5-tailscale-ip>:19999` or `https://app.netdata.cloud`

Tracks CPU, RAM, disk, network, `opcua-howick.service` state. Alerts on crash.

```bash
mise run netdata:install:pi5   # first-time install
```

---

## Hostnames

| Device | mDNS | Tailscale env var |
|--------|------|-------------------|
| Pi Zero 2W | `howick-pi-zero.local` | `ZERO_HOST` |
| Pi 5 | `howick-pi5.local` | `PI5_HOST` |

Use Tailscale IPs after initial setup — mDNS only works on same WiFi segment.

---

## Rebuild all PDFs

```bash
mise run docs:pdf:all   # regenerates docs/dist/01–07.pdf
```

---

**Gerard Webb**
ubuntu Software

