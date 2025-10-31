# srSILO Logging with journald

All srSILO pipeline logs are stored in systemd's journal for centralized querying and automatic rotation.

## 📋 Log Tags

| Tag | Purpose |
|-----|---------|
| `srsilo-pipeline` | Pipeline start/end, main orchestration |
| `srsilo-phase2/3/4/6` | Phase markers (START/COMPLETE) |
| `srsilo-check-data` | check_new_data tool output |
| `srsilo-fetch` | fetch_silo_data tool output |
| `srsilo-split/merge` | Chunk processing output |
| `srsilo-preprocessing` | Docker SILO preprocessing |

## 🔍 Common Commands

```bash
# Pipeline timeline (today)
sudo journalctl -t srsilo-pipeline --since today

# Follow live execution
sudo journalctl -t srsilo-pipeline -f

# All srSILO logs
sudo journalctl SYSLOG_IDENTIFIER=srsilo* --since today

# Specific phase
sudo journalctl -t srsilo-phase2 --since today

# Tool details
sudo journalctl -t srsilo-check-data -n 100
sudo journalctl -t srsilo-fetch --since "1 hour ago"

# Only errors
sudo journalctl -t srsilo-pipeline -p err --since today

# Last 50 entries
sudo journalctl -t srsilo-pipeline -n 50
```

## ⏰ Time Filtering

```bash
--since today              # Today's logs
--since yesterday          # Yesterday only
--since "1 hour ago"       # Last hour
--since "2025-10-31"       # Specific date
--since "14:00"            # Since 2 PM today
-b                         # Current boot
-b -1                      # Previous boot
```

## 🔎 Advanced Usage

```bash
# Search within results
sudo journalctl -t srsilo-phase2 | grep "NEW DATA FOUND"

# JSON output for parsing
sudo journalctl -t srsilo-pipeline -o json --since today

# Short format (no metadata)
sudo journalctl -t srsilo-pipeline -o cat --since today

# Reverse order (newest first)
sudo journalctl -t srsilo-pipeline -n 20 --reverse
```

## 🚨 Troubleshooting

**Pipeline failed?**
```bash
# 1. Check what happened
sudo journalctl -t srsilo-pipeline --since "1 hour ago"

# 2. Find which phase failed
sudo journalctl SYSLOG_IDENTIFIER=srsilo-phase* --since "1 hour ago" -p err

# 3. Get tool details
sudo journalctl -t srsilo-check-data -n 100    # Phase 2
sudo journalctl -t srsilo-fetch -n 100         # Phase 4
sudo journalctl -t srsilo-preprocessing -n 200  # Phase 6
```

**No new data detected?**
```bash
sudo journalctl -t srsilo-phase2 --since today | grep "NEW DATA"
sudo journalctl -t srsilo-check-data -n 50
```

**Memory issues?**
```bash
sudo journalctl --since today | grep -i "out of memory\|oom\|killed"
```

## 💾 Export Logs

```bash
# Save to file
sudo journalctl -t srsilo-pipeline --since today > pipeline.log

# All srSILO logs from specific date
sudo journalctl SYSLOG_IDENTIFIER=srsilo* \
  --since "2025-10-31" --until "2025-11-01" > srsilo-2025-10-31.log

# Copy from remote server
ssh server "sudo journalctl -t srsilo-pipeline --since today" > srsilo.log
```

## ⚙️ Journal Management

```bash
# Check disk usage
sudo journalctl --disk-usage

# Cleanup old logs
sudo journalctl --vacuum-time=30d   # Keep last 30 days
sudo journalctl --vacuum-size=1G    # Keep max 1GB
```

## 🎯 Quick Reference

| Task | Command |
|------|---------|
| Today's pipeline | `sudo journalctl -t srsilo-pipeline --since today` |
| Follow live | `sudo journalctl -t srsilo-pipeline -f` |
| All logs | `sudo journalctl SYSLOG_IDENTIFIER=srsilo* --since today` |
| Only errors | `sudo journalctl -t srsilo-pipeline -p err --since today` |
| Last 50 | `sudo journalctl -t srsilo-pipeline -n 50` |
| Export | `sudo journalctl -t srsilo-pipeline --since today > log.txt` |
