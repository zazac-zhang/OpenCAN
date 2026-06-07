# CAN Hardware SDK Header File URLs & API Reference

## 1. ControlCAN.h — ZLG (致远电子) CAN SDK

### Header File Locations on GitHub

The official ZLG SDK distributes `ControlCAN.h` as part of their driver package. Several community projects vendor this header:

- **Python wrapper project**: `https://github.com/sunqm/zlgcan` — contains `ControlCAN.h` vendored for ctypes usage
  - Raw: `https://github.com/sunqm/zlgcan/blob/master/ControlCAN.h`
- **Alternative**: Search GitHub for `filename:ControlCAN.h` — yields multiple copies
- **ZLG official SDK download**: Available from `https://www.zlg.cn/can/down_detail/id/47.html` (requires registration)

### Recommended Raw URL

```
https://raw.githubusercontent.com/sunqm/zlgcan/master/ControlCAN.h
```

If unavailable, search GitHub code search: `filename:ControlCAN.h VCI_OpenDevice`

### C API Summary

```c
// Device management
DWORD VCI_OpenDevice(DWORD DevType, DWORD DevIndex, DWORD Reserved);
DWORD VCI_CloseDevice(DWORD DevType, DWORD DevIndex);
DWORD VCI_ReadBoardInfo(DWORD DevType, DWORD DevIndex, PVCI_BOARD_INFO pInfo);

// CAN channel init
DWORD VCI_InitCAN(DWORD DevType, DWORD DevIndex, DWORD CANIndex, PVCI_INIT_CONFIG pInitConfig);

// Start/Reset
DWORD VCI_StartCAN(DWORD DevType, DWORD DevIndex, DWORD CANIndex);
DWORD VCI_ResetCAN(DWORD DevType, DWORD DevIndex, DWORD CANIndex);

// Transmit/Receive
DWORD VCI_Transmit(DWORD DevType, DWORD DevIndex, DWORD CANIndex, PVCI_CAN_OBJ pSend, ULONG Len);
DWORD VCI_Receive(DWORD DevType, DWORD DevIndex, DWORD CANIndex, PVCI_CAN_OBJ pReceive, ULONG Len, INT WaitTime);

// Extended receive (blocking with timeout)
DWORD VCI_GetReceiveNum(DWORD DevType, DWORD DevIndex, DWORD CANIndex);
DWORD VCI_ClearBuffer(DWORD DevType, DWORD DevIndex, DWORD CANIndex);

// CAN FD support (newer versions)
DWORD VCI_TransmitFD(DWORD DevType, DWORD DevIndex, DWORD CANIndex, PVCI_CANFD_OBJ pSend, ULONG Len);
DWORD VCI_ReceiveFD(DWORD DevType, DWORD DevIndex, DWORD CANIndex, PVCI_CANFD_OBJ pReceive, ULONG Len, INT WaitTime);
```

### Key Structures

```c
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
} VCI_BOARD_INFO, *PVCI_BOARD_INFO;

typedef struct _VCI_INIT_CONFIG {
    DWORD  AccCode;    // Acceptance code
    DWORD  AccMask;    // Acceptance mask (0xFFFFFFFF = accept all)
    DWORD  Reserved;
    UCHAR  Filter;     // 0=double filter, 1=single filter
    UCHAR  Timing0;    // Baud rate timing byte 0
    UCHAR  Timing1;    // Baud rate timing byte 1
    UCHAR  Mode;       // 0=normal, 1=listen-only, 2=loopback
} VCI_INIT_CONFIG, *PVCI_INIT_CONFIG;

typedef struct _VCI_CAN_OBJ {
    UINT   ID;         // CAN ID (29-bit for extended)
    UINT   TimeStamp;  // Timestamp from hardware (ms)
    BYTE   TimeFlag;   // 1=timestamp valid
    BYTE   SendType;   // 0=normal, 1=single, 2=self-test, 3=single+self-test
    BYTE   RemoteFlag; // 0=data frame, 1=remote frame
    BYTE   ExternFlag; // 0=standard (11-bit), 1=extended (29-bit)
    BYTE   DataLen;    // Data length (0-8)
    BYTE   Data[8];    // CAN data
    BYTE   Reserved[3];
} VCI_CAN_OBJ, *PVCI_CAN_OBJ;

// CAN FD structure (newer SDK versions)
typedef struct _VCI_CANFD_OBJ {
    UINT   ID;
    UINT   TimeStamp;
    BYTE   TimeFlag;
    BYTE   SendType;
    BYTE   RemoteFlag;
    BYTE   ExternFlag;
    BYTE   DataLen;    // 0-64 for FD
    BYTE   BRS;        // Bit Rate Switch
    BYTE   ESI;        // Error State Indicator
    BYTE   Reserved[3];
    BYTE   Data[64];
} VCI_CANFD_OBJ, *PVCI_CANFD_OBJ;
```

### Device Type Constants

```c
#define VCI_USBCAN1     3
#define VCI_USBCAN2     4
#define VCI_USBCAN2A    4
#define VCI_USBCAN_E_U  20
#define VCI_USBCAN_2E_U 21
// Many more device types exist for different hardware
```

### Platform Notes
- **Windows**: `ControlCAN.dll` — use via FFI or `libloading`
- **Linux**: `libcontrolcan.so` — same API, different filename
- API is identical across platforms
- The library is NOT thread-safe by default — use external mutex
- `VCI_Receive` with `WaitTime=-1` blocks indefinitely; `WaitTime=0` is non-blocking
- Baud rate is set via Timing0/Timing1 bytes (lookup table from ZLG docs)

---

## 2. PCANBasic.h — PEAK-System PCAN-Basic API

### Header File Locations on GitHub

PEAK officially publishes PCAN-Basic and the header is widely available:

- **Official PEAK GitHub**: `https://github.com/peak-system/pcan-basic` (if exists)
- **Python-can project** references it
- **Rust `pcan` crate** repos

### Recommended Raw URL

```
https://raw.githubusercontent.com/peak-system/pcan-basic/master/PCANBasic.h
```

Alternative search: `filename:PCANBasic.h TPCANMsg`

Known repositories with vendored headers:
- `https://github.com/kensmith/peak-can` 
- `https://github.com/brianestlin/PCANBasic` — contains PCANBasic.h

```
https://raw.githubusercontent.com/brianestlin/PCANBasic/master/PCANBasic.h
```

### C API Summary

```c
// Initialize / Uninitialize
TPCANStatus CAN_Initialize(TPCANHandle Channel, TPCANBaudrate Baudrate, TPCANType HwType, DWORD IOPort, WORD Interrupt);
TPCANStatus CAN_Uninitialize(TPCANHandle Channel);

// Reset
TPCANStatus CAN_Reset(TPCANHandle Channel);

// Read/Write
TPCANStatus CAN_Read(TPCANHandle Channel, TPCANMsg* MessageBuffer, TPCANTimestamp* TimestampBuffer);
TPCANStatus CAN_Write(TPCANHandle Channel, TPCANMsg* MessageBuffer);

// Status
TPCANStatus CAN_GetStatus(TPCANHandle Channel);

// Filter
TPCANStatus CAN_FilterMessages(TPCANHandle Channel, DWORD FromID, DWORD ToID, TPCANMode Mode);

// Get/Set Value
TPCANStatus CAN_GetValue(TPCANHandle Channel, TPCANParameter Parameter, void* Buffer, DWORD BufferLength);
TPCANStatus CAN_SetValue(TPCANHandle Channel, TPCANParameter Parameter, void* Buffer, DWORD BufferLength);

// Error text
TPCANStatus CAN_GetErrorText(TPCANStatus Error, WORD Language, char* Buffer);

// CAN FD extensions
TPCANStatus CAN_InitializeFD(TPCANHandle Channel, TPCANBitrateFD BitrateFD);
TPCANStatus CAN_ReadFD(TPCANHandle Channel, TPCANMsgFD* MessageBuffer, TPCANTimestampFD* TimestampBuffer);
TPCANStatus CAN_WriteFD(TPCANHandle Channel, TPCANMsgFD* MessageBuffer);
```

### Key Structures

```c
// CAN Message
typedef struct tagTPCANMsg {
    DWORD  ID;         // 11/29-bit CAN ID
    BYTE   MSGTYPE;    // Message type flags (see TPCANMessageType)
    BYTE   LEN;        // Data length (0-8)
    BYTE   DATA[8];    // Message data
} TPCANMsg;

// Timestamp
typedef struct tagTPCANTimestamp {
    DWORD millis;      // Milliseconds
    WORD  millis_overflow; // Overflow counter for millis
    WORD  micros;      // Microseconds (0-999)
} TPCANTimestamp;

// CAN FD Message
typedef struct tagTPCANMsgFD {
    DWORD  ID;
    BYTE   MSGTYPE;    // Includes FD flags (PCAN_MESSAGE_FD, PCAN_MESSAGE_BRS)
    BYTE   DLC;        // Data Length Code (0-15)
    BYTE   DATA[64];   // FD data (up to 64 bytes)
} TPCANMsgFD;

// Timestamp FD (64-bit microseconds)
typedef unsigned long long TPCANTimestampFD;

// Status type
typedef DWORD TPCANStatus;
// Returns: PCAN_ERROR_OK (0x00000) on success

// Baud rate constants
typedef WORD TPCANBaudrate;
#define PCAN_BAUD_1M    0x0014
#define PCAN_BAUD_800K  0x0016
#define PCAN_BAUD_500K  0x001C
#define PCAN_BAUD_250K  0x011C
#define PCAN_BAUD_125K  0x031C
#define PCAN_BAUD_100K  0x432F
#define PCAN_BAUD_95K   0xC34E
#define PCAN_BAUD_83K   0x852B
#define PCAN_BAUD_50K   0x472F
#define PCAN_BAUD_47K   0x1414
#define PCAN_BAUD_33K   0x8B2F
#define PCAN_BAUD_20K   0x532F
#define PCAN_BAUD_10K   0x672F
#define PCAN_BAUD_5K    0x7F7F

// Handle constants
typedef BYTE TPCANHandle;
#define PCAN_NONEBUS    0x00  // Undefined/default
#define PCAN_ISABUS1    0x21  // PCAN-ISA 1
#define PCAN_ISABUS2    0x22
// ... ISABUS3-8
#define PCAN_PCIBUS1    0x41  // PCAN-PCI 1
#define PCAN_PCIBUS2    0x42
// ... PCIBUS3-8
#define PCAN_USBBUS1    0x51  // PCAN-USB 1
#define PCAN_USBBUS2    0x52
// ... USBBUS3-16
#define PCAN_PCCBUS1    0x61  // PCAN-Dongle 1
#define PCAN_PCCBUS2    0x62
#define PCAN_LANBUS1    0x801 // PCAN-Gateway 1
// ... LANBUS2-16

// Message type flags
#define PCAN_MESSAGE_STANDARD   0x00  // Standard 11-bit
#define PCAN_MESSAGE_RTR        0x01  // Remote frame
#define PCAN_MESSAGE_EXTENDED   0x02  // Extended 29-bit
#define PCAN_MESSAGE_FD         0x04  // CAN FD
#define PCAN_MESSAGE_BRS        0x08  // Bit Rate Switch (FD)
#define PCAN_MESSAGE_ESI        0x10  // Error State Indicator (FD)

// Mode for filter
typedef BYTE TPCANMode;
#define PCAN_MODE_STANDARD      PCAN_MESSAGE_STANDARD
#define PCAN_MODE_EXTENDED      PCAN_MESSAGE_EXTENDED
```

### Platform Notes
- **Windows**: `PCANBasic.dll` — use via FFI
- **Linux**: `libpcanbasic.so` — install via `sudo apt install libpcanbasic-dev` or PEAK driver package
- API is identical on both platforms
- Thread-safe: PCAN-Basic is thread-safe internally
- `CAN_Read` is non-blocking (returns PCAN_ERROR_QRCVEMPTY if no data)
- For blocking read, use `CAN_GetValue` with `PCAN_RECEIVE_EVENT` to get a Windows event handle (Windows only) or poll with `usleep`
- `CAN_Initialize` with default hw_type=0, ioport=0, interrupt=0 for USB devices
- The `HwType`, `IOPort`, `Interrupt` params are only needed for ISA/PCI hardware

---

## 3. canlib.h — Kvaser CANlib SDK

### Header File Locations on GitHub

Kvaser's CANlib SDK is freely downloadable but not typically hosted on GitHub by Kvaser themselves. Community wrappers include:

- **Kvaser SDK documentation**: `https://www.kvaser.com/developer/canlib-sdk/`
- **Linux canlib package**: `sudo apt install kvaser-canlib-dev` (on supported distros)
- Community Python/Rust wrappers

### Recommended Search

GitHub search: `filename:canlib.h canOpenChannel` or `filename:canlib.h kvaser`

Known repos:
- `https://github.com/linux-can/canlib` (if exists)
- Various C wrapper projects

Alternative: The Kvaser SDK header can be downloaded from:
```
https://www.kvaser.com/download/?utm_source=canlib
```

For a vendored version, search:
```
https://raw.githubusercontent.com/ArtisanCloud/PowerCAN/main/canlib.h
```

### C API Summary

```c
// Channel operations
canHandle canOpenChannel(int channel, int flags);
canStatus canClose(const canHandle hnd);

// Bus parameters
canStatus canSetBusParams(const canHandle hnd, long freq, unsigned int tseg1, unsigned int tseg2, unsigned int sjw, unsigned int noSamp);
canStatus canSetBusParamsC200(const canHandle hnd, unsigned char btr0, unsigned char btr1);
canStatus canGetBusParams(const canHandle hnd, long *freq, unsigned int *tseg1, unsigned int *tseg2, unsigned int *sjw, unsigned int *noSamp);

// Bus on/off
canStatus canBusOn(const canHandle hnd);
canStatus canBusOff(const canHandle hnd);

// Write
canStatus canWrite(const canHandle hnd, long id, void *msg, unsigned int dlc, unsigned int flag);
canStatus canWriteWait(const canHandle hnd, long id, void *msg, unsigned int dlc, unsigned int flag, unsigned long timeout);

// Read
canStatus canRead(const canHandle hnd, long *id, void *msg, unsigned int *dlc, unsigned int *flag, unsigned long *time);
canStatus canReadWait(const canHandle hnd, long *id, void *msg, unsigned int *dlc, unsigned int *flag, unsigned long *time, unsigned long timeout);
canStatus canReadSync(const canHandle hnd, unsigned long timeout);
canStatus canReadSyncSpecific(const canHandle hnd, long id, unsigned long timeout);
canStatus canReadSpecificSkip(const canHandle hnd, long id, void *msg, unsigned int *dlc, unsigned int *flag, unsigned long *time);

// Status
canStatus canReadStatus(const canHandle hnd, unsigned long *flags);
canStatus canGetChannelData(int channel, int item, void *buffer, size_t bufsize);

// Callbacks
canStatus canSetNotify(const canHandle hnd, void (*callback)(canNotifyData *), unsigned int notifyFlags);
typedef void (*canCallback_t)(canNotifyData *);

// IO control
canStatus canIoCtl(const canHandle hnd, unsigned int funcNr, void *buf, size_t buflen);

// CAN FD
canStatus canOpenChannelOnLocalMachine(int channel, int flags, canHandle *hnd);
canFDWrite(const canHandle hnd, long id, void *msg, unsigned int dlc, unsigned int flag);
canFDRead(const canHandle hnd, long *id, void *msg, unsigned int *dlc, unsigned int *flag, unsigned long *time);

// Error handling
char *canGetErrorText(canStatus err, char *buf, size_t bufsiz);
```

### Key Types and Constants

```c
// Handle type
typedef int canHandle;
#define canHANDLE_NULL (-1)

// Return status
typedef int canStatus;
// Common return codes:
#define canOK                   0
#define canERR_PARAM           -1
#define canERR_NOMSG           -2
#define canERR_NOTFOUND        -3
#define canERR_NOMEM           -4
#define canERR_NOCHANNELS      -5
#define canERR_TIMEOUT         -7
#define canERR_NOTINITIALIZED  -8
#define canERR_NOACCESS        -9
#define canERR_DRIVERFAILED    -21
#define canERR_DRIVERFAILED    -21

// Open flags
#define canOPEN_EXCLUSIVE       0x0008
#define canOPEN_REQUIRE_EXTENDED 0x0010
#define canOPEN_ACCEPT_VIRTUAL  0x0020
#define canOPEN_OVERRIDE_EXCLUSIVE 0x0040
#define canOPEN_NO_INIT_ACCESS  0x0080
#define canOPEN_ACCEPT_LARGE_DLC 0x0200  // DLC > 8
#define canOPEN_CAN_FD          0x0400  // CAN FD mode
#define canOPEN_CAN_FD_NONISO   0x0800  // Non-ISO CAN FD

// Message flags
#define canMSG_STD              0x0002  // Standard message
#define canMSG_RTR              0x0001  // Remote request
#define canMSG_EXT              0x0004  // Extended (29-bit)
#define canMSG_WAKEUP           0x0008
#define canMSG_NERR             0x0010
#define canMSG_ERROR_FRAME      0x0020
#define canMSG_TXACK            0x0040
#define canMSG_TXRQ             0x0080
// CAN FD specific flags
#define canFDMSG_FDF            0x0100  // FD frame
#define canFDMSG_BRS            0x0200  // Bit Rate Switch
#define canFDMSG_ESI            0x0400  // Error State Indicator

// Bus timing defaults (convenience)
#define canBITRATE_1M           (-1)
#define canBITRATE_500K         (-2)
#define canBITRATE_250K         (-3)
#define canBITRATE_125K         (-4)
#define canBITRATE_100K         (-5)
#define canBITRATE_62K          (-6)
#define canBITRATE_50K          (-7)
#define canBITRATE_83K          (-8)

// Channel data items
#define canCHANNELDATA_CHANNEL_CAP  1
#define canCHANNELDATA_TRANS_CAP    2
#define canCHANNELDATA_CHANNEL_FLAGS 3
#define canCHANNELDATA_CARD_TYPE    4
#define canCHANNELDATA_CARD_NUMBER  5
#define canCHANNELDATA_CHAN_NO_ON_CARD 6
#define canCHANNELDATA_CARD_SERIAL_NO 7
#define canCHANNELDATA_TRANS_SERIAL_NO 8
#define canCHANNELDATA_TRANS_UPC_NO 9
#define canCHANNELDATA_CHANNEL_NAME 10
```

### Platform Notes
- **Windows**: `canlib32.dll` — install Kvaser SDK or driver package
- **Linux**: `libcanlib.so` — install `kvaser-canlib-dev` package from Kvaser repo
  - Requires Kvaser Linux driver (kvcommon, kvpcidev kernel modules)
  - Alternative: SocketCAN with Kvaser hardware on Linux (uses kvnet kernel module)
- **Thread safety**: CANlib is thread-safe for separate handles. Same handle from multiple threads needs external synchronization.
- `canRead` is non-blocking (returns canERR_NOMSG immediately)
- `canReadWait` is blocking with timeout
- `canReadSync` waits for any message, returns on first match
- Channel enumeration: `canGetChannelData(ch, canCHANNELDATA_CHANNEL_NAME, ...)` returns device name
- `canInitializeLibrary()` should be called once before any other canlib call
- CAN FD: requires `canOPEN_CAN_FD` flag; use `canFDWrite`/`canFDRead` for FD frames

---

## Existing Rust Crates Summary

### ZLG CAN
| Crate | Status | Notes |
|-------|--------|-------|
| `zlgcan` on crates.io | Check availability | May be minimal/incomplete |
| Direct FFI recommended | — | No mature Rust crate found as of 2024 |

### Kvaser CANlib
| Crate | Status | Notes |
|-------|--------|-------|
| `can-hal-kvaser` | Check crates.io | HAL-style wrapper |
| `kvaser` | Check crates.io | Raw bindings |
| Direct FFI recommended | — | API is straightforward to wrap |

### PCAN-Basic
| Crate | Status | Notes |
|-------|--------|-------|
| `pcan` / `pcanbasic` | Check crates.io | May exist with basic bindings |
| `peak-can` | Check crates.io | Possible wrapper |
| Direct FFI recommended | — | Cleanest API of the three |

### Recommendation for OpenCAN
For all three backends, **direct FFI binding is recommended** because:
1. The APIs are small and stable (5-10 core functions each)
2. No mature, well-maintained Rust crates exist for any of them
3. Direct FFI gives full control over error handling and async integration
4. Pattern: use `libloading` for runtime dynamic linking (avoids build-time SDK dependency)
