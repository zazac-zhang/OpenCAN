# CAN SDK Header Files — Raw GitHub URLs

## Quick Reference

| SDK | Header | Best Raw URL | Search Query |
|-----|--------|-------------|--------------|
| ZLG ControlCAN | ControlCAN.h | `https://raw.githubusercontent.com/sunqm/zlgcan/master/ControlCAN.h` | `filename:ControlCAN.h VCI_OpenDevice` |
| PEAK PCAN-Basic | PCANBasic.h | `https://raw.githubusercontent.com/brianestlin/PCANBasic/master/PCANBasic.h` | `filename:PCANBasic.h TPCANMsg` |
| Kvaser CANlib | canlib.h | Install via package (see below) | `filename:canlib.h canOpenChannel` |

## 1. ControlCAN.h — ZLG

### Primary URL
```
https://raw.githubusercontent.com/sunqm/zlgcan/master/ControlCAN.h
```

### Fallback
```
https://raw.githubusercontent.com/fishpepper/zlgcan/master/ControlCAN.h
```

### GitHub Search
```
https://github.com/search?q=filename%3AControlCAN.h+VCI_OpenDevice&type=code
```

### Official Source
ZLG official SDK download: `https://www.zlg.cn/can/down_detail/id/47.html` (registration required)

### Verification
```bash
curl -sL "https://raw.githubusercontent.com/sunqm/zlgcan/master/ControlCAN.h" | head -30
```

---

## 2. PCANBasic.h — PEAK-System

### Primary URL
```
https://raw.githubusercontent.com/brianestlin/PCANBasic/master/PCANBasic.h
```

### Fallback
Check PEAK official GitHub: `https://github.com/peak-system`
- May have `PCAN-Basic` repo with official header

### GitHub Search
```
https://github.com/search?q=filename%3APCANBasic.h+PCAN_ERROR_OK&type=code
```

### Alternative Repos
- `kensmith/peak-can` — check for PCANBasic.h
- `python-can` project — references header in pcan interface module

### Package Manager
```bash
# Linux
sudo apt install libpcanbasic-dev
# Header at: /usr/include/PCANBasic.h
```

### Verification
```bash
curl -sL "https://raw.githubusercontent.com/brianestlin/PCANBasic/master/PCANBasic.h" | head -30
```

---

## 3. canlib.h — Kvaser

### Best Approach: Package Manager
```bash
# Linux (Debian/Ubuntu)
sudo apt install kvaser-canlib-dev
# Header at: /usr/include/canlib.h

# Or download SDK from:
# https://www.kvaser.com/developer/canlib-sdk/
```

### GitHub Search
```
https://github.com/search?q=filename%3Acanlib.h+canOpenChannel&type=code
```

### Kvaser Official
- GitHub org: `https://github.com/kvaser`
- SDK download: `https://www.kvaser.com/download/`

### Note
canlib.h is rarely vendored on GitHub. The Kvaser SDK must be downloaded or installed via package manager. If you find a GitHub repo with the header, the raw URL pattern is:
```
https://raw.githubusercontent.com/<user>/<repo>/master/canlib.h
```

---

## How to Verify URLs

```bash
# Check if URL returns valid C header
curl -sL "<url>" | head -5
# Should show: #ifndef _CONTROLCAN_H / #include / typedef etc.

# If 404, try alternative search:
# 1. Go to https://github.com/search
# 2. Search: filename:<header>.h <key_function>
# 3. Click result, click "Raw" button
# 4. Copy URL
```

## If All URLs Fail

1. **GitHub Code Search**: Search `filename:<header>.h` with a key function name
2. **Google**: Search `"<header>.h" site:github.com`
3. **Package Manager**: Install SDK packages (headers included)
4. **Vendor Website**: Download official SDK (may require registration)
5. **Python Wrappers**: Often vendor headers for ctypes/cffi (search PyPI)
