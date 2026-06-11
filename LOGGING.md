# Melisa Logging System

## Overview

Melisa memiliki sistem logging yang tangguh dan terinspirasi dari **Nginx**, dirancang untuk production-grade proxy dengan minimal overhead.

### Fitur Utama

✅ **Nginx-style access logging** - Format request/response terstruktur
✅ **Automatic log rotation** - Berdasarkan ukuran file (size-based rotation)
✅ **Configurable log levels** - Debug, Info, Warn, Error
✅ **Buffered writing** - Non-blocking I/O performance
✅ **Separate log files** - Access, Error, Debug, Proxy-specific logs
✅ **Request tracing** - Unique request ID untuk debugging

---

## Directory Structure

```
./logs/
├── access.log      # HTTP request/response logging
├── error.log       # Error dan info messages  
├── debug.log       # Debug-level messages
├── proxy.log       # Proxy events & metrics
├── access.log.20260609_150230    # Rotated backup
└── ...
```

---

## Configuration

Edit `config.toml` untuk mengatur logging:

```toml
[logging]
log_dir = "./logs"
access_log_enabled = true
error_log_enabled = true
debug_log_enabled = false
max_file_size_mb = 100
max_backups = 10
flush_interval_ms = 1000
level = "info"
```

### Parameter Penjelasan

| Parameter | Tipe | Default | Keterangan |
|-----------|------|---------|------------|
| `log_dir` | String | `./logs` | Direktori penyimpanan log |
| `access_log_enabled` | Boolean | `true` | Enable HTTP access log |
| `error_log_enabled` | Boolean | `true` | Enable error log |
| `debug_log_enabled` | Boolean | `false` | Enable debug log |
| `max_file_size_mb` | Integer | `100` | Max ukuran sebelum rotation (MB) |
| `max_backups` | Integer | `10` | Jumlah backup yang di-retain |
| `flush_interval_ms` | Integer | `1000` | Flush buffer interval (ms) |
| `level` | String | `info` | Log level: debug/info/warn/error |

---

## Log Formats

### Access Log (Nginx-style)

Format:
```
127.0.0.1 - - [09/Jun/2026:15:02:30 +0800] "GET /api/users HTTP/1.1" 200 1024 "0ms" "node-1"
```

Contoh:
```
192.168.1.100 - - [09/Jun/2026:15:15:45 +0800] "POST /api/transaction HTTP/1.1" 201 2048 "125ms" "payment-service"
10.0.0.50 - - [09/Jun/2026:15:16:00 +0800] "GET /health HTTP/1.1" 200 256 "5ms" "load-balancer"
```

**Fields:**
- `$remote_addr` - IP address klien
- `$time_local` - Waktu request (format lokal)
- `$request` - Method URI Protocol
- `$status` - HTTP status code (200, 404, 502, dll)
- `$bytes_sent` - Bytes yang dikirim
- `$request_time` - Response time dalam ms
- `$upstream_node` - Nama/ID upstream node yang handle request

### Error Log

Format:
```
[2026/06/09 15:02:30] [ERROR] [REQ-abc123def456] Failed to reach upstream - connection timeout
[2026/06/09 15:02:31] [WARN] No route found for example.com/api/unknown
[2026/06/09 15:02:32] [INFO] Melisa Proxy starting on 127.0.0.1:8080
```

### Debug Log

Format:
```
[2026/06/09 15:02:30.123] [DEBUG] [REQ-abc123def456] Incoming request: GET /api/users from 192.168.1.100
[2026/06/09 15:02:30.150] [DEBUG] [REQ-abc123def456] Route matched -> payment-service (http://127.0.0.1:3000)
[2026/06/09 15:02:30.200] [DEBUG] [REQ-abc123def456] Retry attempt 1/3 for http://127.0.0.1:3000/api/users (reason: timeout)
```

---

## Log Rotation

Sistem log rotation **automatic** berdasarkan **ukuran file**:

### Contoh Rotasi

```
Awalnya:
access.log (45 MB)

Ketika mencapai 100 MB:
access.log (0 bytes - baru)
access.log.20260609_150230 (100 MB - backup)
access.log.20260609_120000 (100 MB - backup lama)
...
access.log.20260608_090000 (100 MB - akan dihapus jika >max_backups)
```

### Cleanup Policy

- Backup yang melebihi `max_backups` akan **otomatis dihapus**
- Backup tertua dihapus terlebih dahulu (FIFO)
- Default: retain 10 backup files

---

## Buffering Strategy (Non-Blocking)

Logger menggunakan **buffering** untuk performa optimal:

1. **Write Buffer** - Log disimpan di memory buffer
2. **Flush Threshold** - Buffer ditulis ke disk:
   - **Setiap** `flush_interval_ms` (default: 1000ms = 1 detik)
   - **Otomatis** ketika buffer penuh
3. **Thread-Safe** - Menggunakan `Arc<Mutex>` untuk thread safety

### Impact

✅ Minimal I/O overhead
✅ Fast request processing
✅ Tidak blocking proxy operation
✅ Automatic flush ke disk

---

## Log Levels

### debug
**Most verbose** - Semua message ditampilkan
```
[DEBUG] Incoming request...
[INFO]  Melisa starting...
[WARN]  No route found...
[ERROR] Connection failed...
```

### info (Default)
Production-grade filtering
```
[INFO]  Melisa starting...
[WARN]  No route found...
[ERROR] Connection failed...
```

### warn
Hanya warning dan error
```
[WARN]  No route found...
[ERROR] Connection failed...
```

### error
**Most strict** - Hanya error
```
[ERROR] Connection failed...
```

---

## Best Practices

### 1. **Monitoring Log Files**

```bash
# Real-time access log monitoring
tail -f logs/access.log

# Count requests per minute
watch 'wc -l logs/access.log'

# Find errors
grep ERROR logs/error.log

# Find slow requests (>1000ms)
grep -E '"[0-9]{4,}ms"' logs/access.log
```

### 2. **Log Rotation Configuration**

```toml
# Production: Aggressive rotation
max_file_size_mb = 50      # Rotate setiap 50 MB
max_backups = 20           # Keep 20 backups (~1 GB total)

# Development: Relaxed rotation
max_file_size_mb = 200     # Rotate setiap 200 MB
max_backups = 5            # Keep 5 backups
```

### 3. **Performance Tuning**

```toml
# High-throughput environment
flush_interval_ms = 2000   # Flush setiap 2 detik (less I/O)

# Low-latency critical
flush_interval_ms = 100    # Flush lebih sering (more I/O)
```

### 4. **Production Settings**

```toml
[logging]
log_dir = "/var/log/melisa"      # Use system log directory
access_log_enabled = true
error_log_enabled = true
debug_log_enabled = false         # Disable debug untuk performance
max_file_size_mb = 100
max_backups = 30
flush_interval_ms = 1000
level = "info"                    # Production level
```

---

## Troubleshooting

### Q: Log files tidak ter-create?
**A:** Pastikan direktori `log_dir` dapat di-write dan permissions terpenuhi
```bash
chmod 755 logs/
```

### Q: Rotation tidak terjadi?
**A:** Cek `max_file_size_mb` setting, pastikan sudah mencapai limit
```toml
max_file_size_mb = 10  # Lower value untuk testing
```

### Q: Performance issues?
**A:** Increase `flush_interval_ms` atau disable `debug_log_enabled`
```toml
flush_interval_ms = 5000    # Flush less frequently
debug_log_enabled = false   # Disable debug logs
```

### Q: Disk space penuh?
**A:** Kurangi `max_backups` atau enable log archiving/compression
```toml
max_backups = 5             # Keep fewer backups
```

---

## Integration Examples

### PHP/Laravel
Baca Melisa access log untuk analytics:
```php
$logs = file('logs/access.log');
foreach ($logs as $line) {
    // Parse dan process...
}
```

### ELK Stack
Ship logs ke Elasticsearch:
```bash
filebeat -c melisa-filebeat.yml
```

### Grafana
Create dashboards dari access log metrics:
```
count() by (status) from access.log
```

---

## API Access Logging

Setiap HTTP request ke Melisa akan ter-log otomatis:

**Example:**
```
Request  → POST /api/users
Response ← 201 Created (250ms)
Log Entry: "192.168.1.100 - - [09/Jun/2026:15:02:30 +0800] "POST /api/users HTTP/1.1" 201 1024 "250ms" "user-service"
```

Request ID (`REQ-xxx`) digunakan untuk tracing across logs.

---

## Summary

- ✅ **Nginx-inspired** - Familiar format untuk ops team
- ✅ **Non-aggressive** - Buffering meminimalkan I/O impact
- ✅ **Production-ready** - Rotation, levels, thread-safe
- ✅ **Configurable** - Sesuaikan dengan kebutuhan
- ✅ **Observable** - Request tracing dan metrics
