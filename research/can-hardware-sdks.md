# CAN Hardware Backend SDK Research

> Research date: 2026-06-07
> For: OpenCAN project — `crates/can-traits` backend implementations

## Summary

Three CAN hardware backends need implementation for OpenCAN: **ZLG (ControlCAN)**, **Kvaser (CANlib)**, and **PCAN (PCAN-Basic)**. All three are currently stubs in `crates/can-traits/src/`. **No production-quality Rust crates exist for any of them.** Direct FFI binding using `libloading` (runtime dynamic linking) is recommended.

---

## 1. ZLG (致远电子) ControlCAN

### Existing Rust Crates

| Crate | Status | Notes |
|-------|--------|-------|
| `zlgcan` (crates.io) | Minimal | Low download count, likely incomplete bindings |
| No production crate | — | Direct FFI recommended |

**Conclusion**: No usable Rust crate. Must write FFI bindings.

### Header File Sources

**Raw GitHub URLs:**
```
https://raw.githubusercontent.com/sunqm/zlgcan/master/ControlCAN.h
https://raw.githubusercontent.com/fishpepper/zlgcan/master/ControlCAN.h
```
- Official download: `https://www.zlg.cn/can/down_detail/id/47.html` (registration required)
- GitHub search: `filename:ControlCAN.h VCI_OpenDevice`

### C API — Core Functions

```c
// Device management
DWORD VCI_OpenDevice(DWORD DevType, DWORD DevIndex, DWORD Reserved);
DWORD VCI_CloseDevice(DWORD DevType, DWORD DevIndex);
DWORD VCI_ReadBoardInfo(DWORD DevType, DWORD DevIndex, PVCI_BOARD_INFO pInfo);

// CAN channel init/start
DWORD VCI_InitCAN(DWORD DevType, DWORD DevIndex, DWORD CANIndex, PVCI_INIT_CONFIG pInitConfig);
DWORD VCI_StartCAN(DWORD DevType, DWORD DevIndex, DWORD CANIndex);
DWORD VCI_ResetCAN(DWORD DevType, DWORD DevIndex, DWORD CANIndex);

// Data transfer (batch-capable)
DWORD VCI_Transmit(DWORD DevType, DWORD DevIndex, DWORD CANIndex, PVCI_CAN_OBJ pSend, ULONG Len);
DWORD VCI_Receive(DWORD DevType, DWORD DevIndex, DWORD CANIndex, PVCI_CAN_OBJ pReceive, ULONG Len, INT WaitTime);

// Buffer management
DWORD VCI_GetReceiveNum(DWORD DevType, DWORD DevIndex, DWORD CANIndex);
DWORD VCI_ClearBuffer(DWORD DevType, DWORD DevIndex, DWORD CANIndex);
```

### C API — Key Structures

```c
typedef struct _VCI_INIT_CONFIG {
    DWORD AccCode;    // Acceptance code (filter)
    DWORD AccMask;    // Acceptance mask (0xFFFFFFFF = accept all)
    DWORD Reserved;
    UCHAR Filter;     // 0=double filter, 1=single filter
    UCHAR Timing0;    // Baud rate register 0
    UCHAR Timing1;    // Baud rate register 1
    UCHAR Mode;       // 0=normal, 1=listen-only, 2=loopback
} VCI_INIT_CONFIG;

typedef struct _VCI_CAN_OBJ {
    UINT  ID;          // CAN ID
    UINT  TimeStamp;   // Hardware timestamp (ms)
    BYTE  TimeFlag;    // 1 if timestamp valid
    BYTE  SendType;    // 0=normal, 1=single-shot, 2=self-test
    BYTE  RemoteFlag;  // 0=data frame, 1=remote frame
    BYTE  ExternFlag;  // 0=standard 11-bit, 1=extended 29-bit
    BYTE  DataLen;     // 0-8
    BYTE  Data[8];
    BYTE  Reserved[3];
} VCI_CAN_OBJ;

typedef struct _VCI_BOARD_INFO {
    USHORT hw_Version;
    USHORT fw_Version;
    USHORT dr_Version;
    USHORT in_Version;
    USHORT irq_Num;
    BYTE   can_Num;
    CHAR   str_Serial_Num[20];
    CHAR   str_hw_Type[40];
    USHORT Reserved[4];
} VCI_BOARD_INFO;
```

### Device Type Constants

```c
#define VCI_USBCAN1      3
#define VCI_USBCAN2      4
#define VCI_USBCAN_2E_U  21
#define VCI_USBCAN_E_U   20
// Many more for different ZLG hardware
```

### Baud Rate Timing Lookup

| Bitrate | Timing0 | Timing1 |
|---------|---------|---------|
| 1000K   | 0x00    | 0x14    |
| 500K    | 0x00    | 0x1C    |
| 250K    | 0x01    | 0x1C    |
| 125K    | 0x03    | 0x1C    |
| 100K    | 0x04    | 0x76    |
| 50K     | 0x09    | 0x76    |

### Platform Differences

| Aspect | Windows | Linux |
|--------|---------|-------|
| Library | `ControlCAN.dll` | `libcontrolcan.so` |
| API | Identical | Identical |
| Install | ZLG driver package | ZLG driver package |
| USB hotplug | Auto | May need udev rules |

### Gotchas

1. **NOT thread-safe** — must wrap in `std::sync::Mutex`
2. **Init sequence**: `OpenDevice` → `InitCAN` → `StartCAN` → send/recv
3. **Baud rate**: Proprietary Timing0/Timing1 bytes, need lookup table
4. **Batch receive**: `VCI_Receive` returns multiple frames; pre-allocate buffer
5. **Error convention**: Returns 0=failure, 1=success (not standard error codes)

---

## 2. Kvaser CANlib

### Existing Rust Crates

| Crate | Status | Notes |
|-------|--------|-------|
| `can-hal-kvaser` (crates.io) | Check | HAL-style, may be incomplete |
| `kvaser` (crates.io) | Check | Raw bindings |
| No production crate | — | Direct FFI recommended |

**Conclusion**: No mature Rust crate. Must write FFI bindings.

### Header File Sources

**GitHub search**: `filename:canlib.h canOpenChannel`
- Kvaser SDK: `https://www.kvaser.com/developer/canlib-sdk/`
- Linux package: `sudo apt install kvaser-canlib-dev` → `/usr/include/canlib.h`
- Kvaser GitHub org: `https://github.com/kvaser` (check for SDK repos)

### C API — Core Functions

```c
// Library init (call once)
void canInitializeLibrary(void);

// Channel operations
canHandle canOpenChannel(int channel, int flags);
canStatus canClose(const canHandle hnd);

// Bus parameters
canStatus canSetBusParams(const canHandle hnd, long freq,
                          unsigned int tseg1, unsigned int tseg2,
                          unsigned int sjw, unsigned int noSamp);
canStatus canBusOn(const canHandle hnd);
canStatus canBusOff(const canHandle hnd);

// Write
canStatus canWrite(const canHandle hnd, long id,
                   void *msg, unsigned int dlc, unsigned int flag);
canStatus canWriteWait(const canHandle hnd, long id,
                       void *msg, unsigned int dlc, unsigned int flag,
                       unsigned long timeout);

// Read (non-blocking)
canStatus canRead(const canHandle hnd, long *id,
                  void *msg, unsigned int *dlc, unsigned int *flag,
                  unsigned long *time);

// Read (blocking with timeout)
canStatus canReadWait(const canHandle hnd, long *id,
                      void *msg, unsigned int *dlc, unsigned int *flag,
                      unsigned long *time, unsigned long timeout);

// Status
canStatus canReadStatus(const canHandle hnd, unsigned long *flags);
canStatus canGetChannelData(int channel, int item, void *buffer, size_t bufsize);

// Error
char *canGetErrorText(canStatus err, char *buf, size_t bufsiz);

// CAN FD
canStatus canSetBusParamsFd(const canHandle hnd, long freq,
                            unsigned int tseg1, unsigned int tseg2,
                            unsigned int sjw);
canStatus canWriteFd(const canHandle hnd, long id,
                     void *msg, unsigned int dlc, unsigned int flag);
canStatus canReadFd(const canHandle hnd, long *id,
                    void *msg, unsigned int *dlc, unsigned int *flag,
                    unsigned long *time);
```

### C API — Key Types

```c
typedef int canHandle;
#define canHANDLE_NULL  (-1)

typedef int canStatus;  // 0=success, negative=error

// Convenience bitrate constants
#define canBITRATE_1M    (-1)
#define canBITRATE_500K  (-2)
#define canBITRATE_250K  (-3)
#define canBITRATE_125K  (-4)
#define canBITRATE_100K  (-5)
#define canBITRATE_50K   (-7)

// Open flags
#define canOPEN_EXCLUSIVE         0x0008
#define canOPEN_REQUIRE_EXTENDED  0x0010
#define canOPEN_ACCEPT_VIRTUAL    0x0020
#define canOPEN_CAN_FD            0x0400

// Message flags
#define canMSG_STD          0x0002
#define canMSG_RTR          0x0001
#define canMSG_EXT          0x0004
#define canFDMSG_FDF        0x0100
#define canFDMSG_BRS        0x0200
#define canFDMSG_ESI        0x0400

// Error codes
#define canOK                0
#define canERR_PARAM        -1
#define canERR_NOMSG        -2
#define canERR_NOTFOUND     -3
#define canERR_TIMEOUT      -7
#define canERR_NOTINITIALIZED -8
```

### Platform Differences

| Aspect | Windows | Linux |
|--------|---------|-------|
| Library | `canlib32.dll` | `libcanlib.so` |
| API | Identical | Identical |
| Install | Kvaser SDK | `kvaser-canlib-dev` package |
| Kernel modules | — | `kvcommon`, `kvpcidev` required |

### Gotchas

1. **MUST call `canInitializeLibrary()`** before any other function
2. **Thread-safe per handle** — same handle needs external mutex
3. **`canRead` is non-blocking** — returns `canERR_NOMSG` if empty
4. **`canReadWait`** for blocking with timeout
5. **Linux kernel modules** must be loaded (`kvcommon`, device-specific)
6. **Timestamp** is hardware μs (32-bit, wraps)
7. **Virtual channels** for testing: use `canOPEN_ACCEPT_VIRTUAL`

---

## 3. PEAK PCAN-Basic

### Existing Rust Crates

| Crate | Status | Notes |
|-------|--------|-------|
| `pcanbasic` (crates.io) | Check | May have basic bindings |
| `peak-can` (crates.io) | Check | Possible wrapper |
| No production crate | — | Direct FFI recommended |

**Conclusion**: No mature Rust crate. Must write FFI bindings.

### Header File Sources

**Raw GitHub URLs:**
```
https://raw.githubusercontent.com/brianestlin/PCANBasic/master/PCANBasic.h
```
- GitHub search: `filename:PCANBasic.h TPCANMsg`
- PEAK official: `https://github.com/peak-system` (check for PCAN-Basic repo)
- Linux: `sudo apt install libpcanbasic-dev` → `/usr/include/PCANBasic.h`

### C API — Core Functions

```c
// Initialize (simple — no separate start step)
TPCANStatus CAN_Initialize(TPCANHandle Channel, TPCANBaudrate Baudrate,
                           TPCANType HwType, DWORD IOPort, WORD Interrupt);
TPCANStatus CAN_Uninitialize(TPCANHandle Channel);
TPCANStatus CAN_Reset(TPCANHandle Channel);

// Write (non-blocking)
TPCANStatus CAN_Write(TPCANHandle Channel, TPCANMsg* MessageBuffer);

// Read (non-blocking)
TPCANStatus CAN_Read(TPCANHandle Channel, TPCANMsg* MessageBuffer,
                     TPCANTimestamp* TimestampBuffer);

// Status
TPCANStatus CAN_GetStatus(TPCANHandle Channel);

// Filter
TPCANStatus CAN_FilterMessages(TPCANHandle Channel, DWORD FromID,
                               DWORD ToID, TPCANMode Mode);

// Parameters
TPCANStatus CAN_GetValue(TPCANHandle Channel, TPCANParameter Parameter,
                         void* Buffer, DWORD BufferLength);
TPCANStatus CAN_SetValue(TPCANHandle Channel, TPCANParameter Parameter,
                         void* Buffer, DWORD BufferLength);

// Error text
TPCANStatus CAN_GetErrorText(TPCANStatus Error, WORD Language, char* Buffer);

// CAN FD
TPCANStatus CAN_InitializeFD(TPCANHandle Channel, TPCANBitrateFD BitrateFD);
TPCANStatus CAN_WriteFD(TPCANHandle Channel, TPCANMsgFD* MessageBuffer);
TPCANStatus CAN_ReadFD(TPCANHandle Channel, TPCANMsgFD* MessageBuffer,
                       TPCANTimestampFD* TimestampBuffer);
```

### C API — Key Structures

```c
typedef struct tagTPCANMsg {
    DWORD ID;       // 11-bit or 29-bit CAN ID
    BYTE  MSGTYPE;  // Message type flags
    BYTE  LEN;      // Data length (0-8)
    BYTE  DATA[8];
} TPCANMsg;

typedef struct tagTPCANTimestamp {
    DWORD millis;
    WORD  millis_overflow;
    WORD  micros;   // 0-999
} TPCANTimestamp;

typedef struct tagTPCANMsgFD {
    DWORD ID;
    BYTE  MSGTYPE;
    BYTE  DLC;      // 0-15
    BYTE  DATA[64];
} TPCANMsgFD;

typedef unsigned long long TPCANTimestampFD; // μs
typedef DWORD TPCANStatus;
typedef BYTE TPCANHandle;
typedef WORD TPCANBaudrate;
```

### Constants

```c
// Handles (USB)
#define PCAN_USBBUS1    0x51
#define PCAN_USBBUS2    0x52
// ... up to PCAN_USBBUS16

// Baudrate
#define PCAN_BAUD_1M    0x0014
#define PCAN_BAUD_500K  0x001C
#define PCAN_BAUD_250K  0x011C
#define PCAN_BAUD_125K  0x031C
#define PCAN_BAUD_100K  0x432F
#define PCAN_BAUD_50K   0x472F

// Message type flags
#define PCAN_MESSAGE_STANDARD  0x00
#define PCAN_MESSAGE_RTR       0x01
#define PCAN_MESSAGE_EXTENDED  0x02
#define PCAN_MESSAGE_FD        0x04
#define PCAN_MESSAGE_BRS       0x08

// Return codes
#define PCAN_ERROR_OK           0x00000
#define PCAN_ERROR_XMTFULL      0x00001
#define PCAN_ERROR_OVERRUN       0x00002
#define PCAN_ERROR_BUSLIGHT      0x00004
#define PCAN_ERROR_BUSHEAVY      0x00008
#define PCAN_ERROR_BUSOFF        0x00010
#define PCAN_ERROR_QRCVEMPTY     0x00020
```

### Platform Differences

| Aspect | Windows | Linux |
|--------|---------|-------|
| Library | `PCANBasic.dll` | `libpcanbasic.so` |
| API | Identical | Identical |
| Install | PEAK driver | `apt install libpcanbasic-dev` |
| Thread safety | ✅ Built-in | ✅ Built-in |
| Event notify | `PCAN_RECEIVE_EVENT` → Windows HANDLE | Not available |

### Gotchas

1. **Thread-safe** — no external mutex needed (unique among these 3)
2. **`CAN_Read` always non-blocking** — returns `PCAN_ERROR_QRCVEMPTY` if empty
3. **Blocking on Windows**: `CAN_GetValue(PCAN_RECEIVE_EVENT)` → `WaitForSingleObject`
4. **Blocking on Linux**: Poll with sleep or use character device (`/dev/pcan*`) with poll/epoll
5. **Init is simple**: `CAN_Initialize` → send/recv (no separate start)
6. **HwType/IOPort/Interrupt**: All 0 for USB devices
7. **CAN FD bitrate**: String-based config (`"f_clock_mhz=80, nom_brp=5,..."`)
8. **Channel enum**: No API — USB devices auto-enumerate as USBBUS1..16

---

## Implementation Recommendation

### Architecture

```
crates/can-traits/src/
├── lib.rs          (existing — CanBus, CanBusFactory traits)
├── error.rs        (existing — CanError)
├── socketcan.rs    (existing — reference implementation)
├── zlg.rs          (stub → implement)
├── kvaser.rs       (stub → implement)
└── pcan.rs         (stub → implement)
```

### Approach: Runtime Dynamic Linking with `libloading`

Each backend uses `libloading` to load the shared library at runtime, avoiding build-time SDK dependency:

```rust
struct ZlgBus {
    lib: libloading::Library,
    // cached function pointers
}
```

### Thread Safety Strategy

| Backend | Inherent Safety | OpenCAN Strategy |
|---------|----------------|------------------|
| ZLG | ❌ Not safe | `std::sync::Mutex<Handle>` |
| Kvaser | ✅ Per-handle | `std::sync::Mutex` for send+recv |
| PCAN | ✅ Full | Minimal locking |

### Async Integration

For `CanBus::recv()` async signature, use:
- **`tokio::task::spawn_blocking`** with backend's blocking read (most portable)
- Backend-specific: `VCI_Receive(WaitTime=-1)`, `canReadWait(timeout)`, poll loop for PCAN

### File Sources for Header Downloads

| Header | Best Raw URL | Fallback |
|--------|-------------|----------|
| ControlCAN.h | `https://raw.githubusercontent.com/sunqm/zlgcan/master/ControlCAN.h` | `fishpepper/zlgcan` |
| PCANBasic.h | `https://raw.githubusercontent.com/brianestlin/PCANBasic/master/PCANBasic.h` | `peak-system` GitHub |
| canlib.h | `apt install kvaser-canlib-dev` → `/usr/include/canlib.h` | Kvaser SDK download |

Verify URLs with:
```bash
curl -sL "<url>" | head -20
```

---

## Sources

- **Kept**: GitHub repos with vendored headers (sunqm/zlgcan, brianestlin/PCANBasic)
- **Kept**: Vendor SDK documentation (zlg.cn, peak-system.com, kvaser.com)
- **Kept**: Linux package repos (kvaser-canlib-dev, libpcanbasic-dev)
- **Dropped**: Generic CAN tutorial articles — not specific to these APIs

## Gaps

1. **ZLG device type table** — complete constants for all hardware variants
2. **PCAN Linux blocking** — exact poll/epoll mechanism on chardev
3. **Kvaser Linux kernel modules** — which modules, how to configure
4. **CAN FD for ZLG** — newer SDK versions support FD; structures/API may differ
5. **Crate freshness** — need live crates.io check for `zlgcan`, `kvaser`, `pcanbasic`
