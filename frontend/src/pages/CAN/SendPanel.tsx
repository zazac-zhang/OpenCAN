/**
 * SendPanel — CAN frame send panel with raw frame and SDO quick access.
 *
 * Features:
 * - Raw frame send (COB-ID, DLC, data with Hex/ASCII/DEC toggle)
 * - Send history with quick-replay
 * - Cyclic send list
 * - SDO Quick Access (node selector, index/sub, read/write)
 */

import { Play, Plus, RotateCcw, Send, Square, Trash2 } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useSdoDownload, useSdoUpload, useSendFrame } from '@/hooks/useCommands';
import { useAppStore, useConnected, useNodes } from '@/lib/store';

type DataFormat = 'hex' | 'ascii' | 'dec';

interface SendHistoryEntry {
  cobId: number;
  dlc: number;
  data: number[];
}

interface CyclicSendEntry {
  cobId: number;
  dlc: number;
  data: number[];
  intervalMs: number;
  running: boolean;
}

export function SendPanel() {
  const connected = useConnected();
  const nodes = useNodes();

  // Raw frame state
  const [cobId, setCobId] = useState('0x181');
  const [dlc, setDlc] = useState(8);
  const [dataHex, setDataHex] = useState('00 00 00 00 00 00 00 00');
  const [dataFormat, setDataFormat] = useState<DataFormat>('hex');
  const [sendHistory, setSendHistory] = useState<SendHistoryEntry[]>([]);
  const [cyclicSends, setCyclicSends] = useState<CyclicSendEntry[]>([]);

  // SDO state
  const [sdoNode, setSdoNode] = useState(1);
  const [sdoIndex, setSdoIndex] = useState('0x1000');
  const [sdoSubindex, setSdoSubindex] = useState('0');
  const [sdoValue, setSdoValue] = useState('');
  const [sdoResult, setSdoResult] = useState<string | null>(null);

  const sendFrameMutation = useSendFrame();
  const sdoUploadMutation = useSdoUpload();
  const sdoDownloadMutation = useSdoDownload();

  // Cyclic send intervals
  const cyclicIntervals = useRef<Map<number, ReturnType<typeof setInterval>>>(new Map());

  // Parse data from hex string
  const parseData = useCallback(
    (hex: string): number[] => {
      return hex
        .trim()
        .split(/\s+/)
        .map((b) => parseInt(b, 16))
        .filter((b) => !Number.isNaN(b) && b >= 0 && b <= 255)
        .slice(0, dlc);
    },
    [dlc],
  );

  // Format data to display
  const formatData = useCallback((bytes: number[]): string => {
    return bytes.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');
  }, []);

  const handleSend = useCallback(() => {
    const cobNum = parseInt(cobId.replace('0x', ''), 16);
    if (Number.isNaN(cobNum) || cobNum < 0 || cobNum > 0x7ff) return;

    const data = parseData(dataHex);
    const frameData = [...data, ...Array(Math.max(0, dlc - data.length)).fill(0)];

    sendFrameMutation.mutate({ cobId: cobNum, data: frameData });

    // Add to history
    setSendHistory((prev) => {
      const entry: SendHistoryEntry = { cobId: cobNum, dlc: frameData.length, data: frameData };
      // Avoid duplicates at the top
      if (
        prev.length > 0 &&
        prev[0].cobId === entry.cobId &&
        prev[0].data.join() === entry.data.join()
      ) {
        return prev;
      }
      return [entry, ...prev].slice(0, 20);
    });
  }, [cobId, dataHex, dlc, parseData, sendFrameMutation]);

  const handleResend = useCallback(
    (entry: SendHistoryEntry) => {
      sendFrameMutation.mutate({ cobId: entry.cobId, data: entry.data });
    },
    [sendFrameMutation],
  );

  const handleAddCyclic = useCallback(() => {
    const cobNum = parseInt(cobId.replace('0x', ''), 16);
    if (Number.isNaN(cobNum)) return;
    const data = parseData(dataHex);
    setCyclicSends((prev) => [
      ...prev,
      { cobId: cobNum, dlc: Math.min(data.length, dlc), data, intervalMs: 100, running: false },
    ]);
  }, [cobId, dataHex, dlc, parseData]);

  const handleToggleCyclic = useCallback((index: number) => {
    setCyclicSends((prev) =>
      prev.map((entry, i) => (i === index ? { ...entry, running: !entry.running } : entry)),
    );
  }, []);

  const handleRemoveCyclic = useCallback((index: number) => {
    // Clear interval if running
    const interval = cyclicIntervals.current.get(index);
    if (interval) {
      clearInterval(interval);
      cyclicIntervals.current.delete(index);
    }
    setCyclicSends((prev) => prev.filter((_, i) => i !== index));
  }, []);

  // Start/stop cyclic send intervals
  useEffect(() => {
    cyclicSends.forEach((entry, index) => {
      const existing = cyclicIntervals.current.get(index);
      if (entry.running && !existing) {
        // Start sending
        const interval = setInterval(() => {
          sendFrameMutation.mutate({ cobId: entry.cobId, data: entry.data });
        }, entry.intervalMs);
        cyclicIntervals.current.set(index, interval);
      } else if (!entry.running && existing) {
        // Stop sending
        clearInterval(existing);
        cyclicIntervals.current.delete(index);
      }
    });

    // Cleanup on unmount
    return () => {
      cyclicIntervals.current.forEach((interval) => clearInterval(interval));
      cyclicIntervals.current.clear();
    };
  }, [cyclicSends, sendFrameMutation]);

  const handleSdoRead = useCallback(() => {
    const idx = parseInt(sdoIndex.replace('0x', ''), 16) || 0;
    const sub = parseInt(sdoSubindex, 10) || 0;
    sdoUploadMutation.mutate(
      { node_id: sdoNode, index: idx, subindex: sub, data_type: 'UNS32' },
      {
        onSuccess: (data) =>
          setSdoResult(data?.data.map((b) => b.toString(16).padStart(2, '0')).join(' ') || '—'),
        onError: () => setSdoResult('Error'),
      },
    );
  }, [sdoNode, sdoIndex, sdoSubindex, sdoUploadMutation]);

  const handleSdoWrite = useCallback(() => {
    const idx = parseInt(sdoIndex.replace('0x', ''), 16) || 0;
    const sub = parseInt(sdoSubindex, 10) || 0;
    const bytes = sdoValue
      .split(' ')
      .filter(Boolean)
      .map((b) => parseInt(b, 16))
      .filter((b) => !Number.isNaN(b));
    sdoDownloadMutation.mutate(
      { node_id: sdoNode, index: idx, subindex: sub, data: bytes },
      {
        onSuccess: () => setSdoResult('OK'),
        onError: () => setSdoResult('Error'),
      },
    );
  }, [sdoNode, sdoIndex, sdoSubindex, sdoValue, sdoDownloadMutation]);

  // Format helpers
  const formatCobId = (cob: number) => `0x${cob.toString(16).padStart(3, '0').toUpperCase()}`;

  return (
    <div className="flex flex-col h-full">
      {/* Frame table (top portion) */}
      <div className="flex-1 overflow-hidden min-h-0">
        <MiniFrameTable />
      </div>

      {/* Resizable divider */}
      <div className="h-1 bg-border/50 hover:bg-primary/30 cursor-row-resize shrink-0" />

      {/* Send panel (bottom portion) */}
      <div className="flex flex-col shrink-0 border-t bg-card/30" style={{ height: '35%' }}>
        {/* Section header */}
        <div className="flex items-center gap-2 px-3 py-1.5 border-b border-border/50 bg-card/50">
          <Send className="w-3.5 h-3.5 text-muted-foreground" />
          <span className="text-xs font-semibold">Send</span>
        </div>

        <div className="flex flex-1 overflow-hidden">
          {/* Left: Raw frame + SDO */}
          <div className="flex-1 flex flex-col overflow-auto min-w-0">
            {/* Raw frame send */}
            <div className="p-3 space-y-2 border-b border-border/50">
              <div className="flex items-center gap-2">
                <span className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
                  Raw Frame
                </span>
              </div>
              <div className="flex items-center gap-2">
                <div className="flex items-center gap-1">
                  <span className="text-[10px] text-muted-foreground w-10">COB-ID</span>
                  <input
                    className="w-16 px-1.5 py-0.5 text-xs font-mono rounded border border-border bg-background"
                    value={cobId}
                    onChange={(e) => setCobId(e.target.value)}
                    placeholder="0x181"
                  />
                </div>
                <div className="flex items-center gap-1">
                  <span className="text-[10px] text-muted-foreground w-6">DLC</span>
                  <select
                    className="w-12 px-1 py-0.5 text-xs rounded border border-border bg-background"
                    value={dlc}
                    onChange={(e) => setDlc(parseInt(e.target.value, 10))}
                  >
                    {[0, 1, 2, 3, 4, 5, 6, 7, 8].map((n) => (
                      <option key={n} value={n}>
                        {n}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
              <div>
                <span className="text-[10px] text-muted-foreground">Data</span>
                <input
                  className="w-full mt-1 px-1.5 py-0.5 text-xs font-mono rounded border border-border bg-background"
                  value={dataHex}
                  onChange={(e) => setDataHex(e.target.value)}
                  placeholder="00 00 00 00 00 00 00 00"
                />
              </div>
              <div className="flex items-center gap-2">
                {(['hex', 'ascii', 'dec'] as DataFormat[]).map((fmt) => (
                  <button
                    key={fmt}
                    className={`px-2 py-0.5 text-[10px] rounded border transition-colors ${
                      dataFormat === fmt
                        ? 'bg-primary text-primary-foreground border-primary'
                        : 'border-border text-muted-foreground hover:bg-muted'
                    }`}
                    onClick={() => setDataFormat(fmt)}
                  >
                    {fmt.toUpperCase()}
                  </button>
                ))}
                <div className="flex-1" />
                <button
                  className="flex items-center gap-1 px-3 py-1 text-xs bg-primary text-primary-foreground rounded hover:bg-primary/90 disabled:opacity-50"
                  onClick={handleSend}
                  disabled={sendFrameMutation.isPending || !connected}
                >
                  <Send className="w-3 h-3" />
                  Send
                </button>
              </div>
            </div>

            {/* SDO Quick Access */}
            <div className="p-3 space-y-2">
              <div className="flex items-center gap-2">
                <span className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
                  SDO Quick Access
                </span>
              </div>
              <div className="flex items-center gap-2">
                <select
                  className="w-16 px-1.5 py-0.5 text-xs rounded border border-border bg-background"
                  value={sdoNode}
                  onChange={(e) => setSdoNode(parseInt(e.target.value, 10))}
                >
                  {nodes.length > 0
                    ? nodes.map((n) => (
                        <option key={n.node_id} value={n.node_id}>
                          Node {n.node_id}
                        </option>
                      ))
                    : [1, 2, 3, 4, 5].map((n) => (
                        <option key={n} value={n}>
                          Node {n}
                        </option>
                      ))}
                </select>
                <input
                  className="w-16 px-1.5 py-0.5 text-xs font-mono rounded border border-border bg-background"
                  value={sdoIndex}
                  onChange={(e) => setSdoIndex(e.target.value)}
                  placeholder="0x1000"
                />
                <span className="text-[10px] text-muted-foreground">:</span>
                <input
                  className="w-10 px-1.5 py-0.5 text-xs font-mono rounded border border-border bg-background"
                  value={sdoSubindex}
                  onChange={(e) => setSdoSubindex(e.target.value)}
                  placeholder="0"
                />
              </div>
              <div className="flex items-center gap-2">
                <button
                  className="px-3 py-1 text-xs bg-primary text-primary-foreground rounded hover:bg-primary/90 disabled:opacity-50"
                  onClick={handleSdoRead}
                  disabled={sdoUploadMutation.isPending || !connected}
                >
                  Read
                </button>
                <input
                  className="flex-1 px-1.5 py-0.5 text-xs font-mono rounded border border-border bg-background"
                  value={sdoValue}
                  onChange={(e) => setSdoValue(e.target.value)}
                  placeholder="hex value (e.g. FF 00 01)"
                />
                <button
                  className="px-3 py-1 text-xs border border-border rounded hover:bg-muted disabled:opacity-50"
                  onClick={handleSdoWrite}
                  disabled={sdoDownloadMutation.isPending || !connected}
                >
                  Write
                </button>
              </div>
              {sdoResult && (
                <div className="text-[10px] font-mono">
                  <span className="text-muted-foreground">Result: </span>
                  <span className={sdoResult === 'Error' ? 'text-red-400' : 'text-green-400'}>
                    {sdoResult}
                  </span>
                </div>
              )}
            </div>
          </div>

          {/* Right: History + Cyclic */}
          <div className="w-64 border-l border-border/50 flex flex-col overflow-hidden">
            {/* Send History */}
            <div className="flex-1 overflow-auto min-h-0">
              <div className="px-2 py-1.5 border-b border-border/50 bg-card/50 flex items-center justify-between">
                <span className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
                  History
                </span>
                {sendHistory.length > 0 && (
                  <button
                    className="p-0.5 text-muted-foreground hover:text-destructive transition-colors"
                    onClick={() => setSendHistory([])}
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                )}
              </div>
              {sendHistory.length === 0 ? (
                <div className="p-4 text-center text-[10px] text-muted-foreground">
                  No frames sent yet
                </div>
              ) : (
                <div className="divide-y divide-border/50">
                  {sendHistory.map((entry, i) => (
                    <div
                      key={i}
                      className="flex items-center gap-1.5 px-2 py-1 text-[10px] font-mono hover:bg-muted/30 transition-colors"
                    >
                      <span className="w-12">{formatCobId(entry.cobId)}</span>
                      <span className="flex-1 truncate">{formatData(entry.data)}</span>
                      <button
                        className="p-0.5 text-muted-foreground hover:text-primary transition-colors"
                        onClick={() => handleResend(entry)}
                        title="Resend"
                      >
                        <RotateCcw className="w-3 h-3" />
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Cyclic Send */}
            <div className="border-t border-border/50 max-h-40 overflow-auto">
              <div className="px-2 py-1.5 border-b border-border/50 bg-card/50 flex items-center justify-between">
                <span className="text-[10px] font-medium text-muted-foreground uppercase tracking-wider">
                  Cyclic
                </span>
                <button
                  className="p-0.5 text-muted-foreground hover:text-primary transition-colors"
                  onClick={handleAddCyclic}
                  title="Add cyclic send"
                >
                  <Plus className="w-3 h-3" />
                </button>
              </div>
              {cyclicSends.length === 0 ? (
                <div className="p-3 text-center text-[10px] text-muted-foreground">
                  Click + to add cyclic send
                </div>
              ) : (
                <div className="divide-y divide-border/50">
                  {cyclicSends.map((entry, i) => (
                    <div
                      key={i}
                      className="flex items-center gap-1 px-2 py-1 text-[10px] font-mono"
                    >
                      <span className="w-12">{formatCobId(entry.cobId)}</span>
                      <span className="w-8 text-muted-foreground">{entry.intervalMs}ms</span>
                      <div className="flex-1" />
                      <button
                        className={`p-0.5 transition-colors ${
                          entry.running
                            ? 'text-green-400 hover:text-green-300'
                            : 'text-muted-foreground hover:text-primary'
                        }`}
                        onClick={() => handleToggleCyclic(i)}
                      >
                        {entry.running ? (
                          <Square className="w-3 h-3" />
                        ) : (
                          <Play className="w-3 h-3" />
                        )}
                      </button>
                      <button
                        className="p-0.5 text-muted-foreground hover:text-destructive transition-colors"
                        onClick={() => handleRemoveCyclic(i)}
                      >
                        <Trash2 className="w-3 h-3" />
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Mini frame table — compact version for the top portion of Send panel
function MiniFrameTable() {
  const frames = useAppStore((s) => s.frames.frames);
  const displayFrames = frames.slice(-200).reverse();

  const formatData = (bytes: number[]) =>
    bytes.map((b) => b.toString(16).padStart(2, '0').toUpperCase()).join(' ');

  const formatTime = (ms: number) => {
    const sec = Math.floor(ms / 1000);
    const millis = ms % 1000;
    return `${sec}.${millis.toString().padStart(3, '0')}`;
  };

  const getTypeLabel = (cobId: number): { label: string; color: string } => {
    if (cobId === 0x080) return { label: 'SYNC', color: 'text-purple-400' };
    if (cobId >= 0x700 && cobId <= 0x77f) return { label: 'HB', color: 'text-green-400' };
    if (cobId >= 0x180 && cobId <= 0x1ff) return { label: 'TPDO1', color: 'text-blue-400' };
    if (cobId >= 0x200 && cobId <= 0x27f) return { label: 'RPDO1', color: 'text-orange-400' };
    if (cobId >= 0x280 && cobId <= 0x2ff) return { label: 'TPDO2', color: 'text-blue-400' };
    if (cobId >= 0x300 && cobId <= 0x37f) return { label: 'RPDO2', color: 'text-orange-400' };
    if (cobId >= 0x580 && cobId <= 0x5ff) return { label: 'SDO', color: 'text-yellow-400' };
    if (cobId >= 0x600 && cobId <= 0x67f) return { label: 'SDO', color: 'text-yellow-400' };
    if (cobId >= 0x081 && cobId <= 0x0ff) return { label: 'NMT', color: 'text-pink-400' };
    if (cobId >= 0x80 && cobId <= 0x7f) return { label: 'NMT', color: 'text-pink-400' };
    return { label: '—', color: 'text-muted-foreground' };
  };

  return (
    <div className="flex flex-col h-full">
      {/* Compact filter bar */}
      <div className="flex items-center gap-2 px-3 py-1 border-b bg-card shrink-0">
        <span className="text-xs font-semibold flex items-center gap-1.5">
          <span className="w-2 h-2 rounded-full bg-blue-500" />
          Frames
        </span>
        <span className="text-[10px] text-muted-foreground ml-auto">
          {frames.length} total · showing last {displayFrames.length}
        </span>
        <button
          onClick={() => useAppStore.getState().frames.clearFrames()}
          className="px-2 py-0.5 text-[10px] rounded border border-red-500/30 text-red-400 hover:bg-red-500/10"
        >
          Clear
        </button>
      </div>

      {/* Table header */}
      <div className="flex items-center gap-2 px-3 py-0.5 bg-muted/50 text-[10px] font-medium border-b shrink-0">
        <span className="w-14">Time</span>
        <span className="w-16">COB-ID</span>
        <span className="w-12">Type</span>
        <span className="w-6">Dir</span>
        <span className="w-6">DLC</span>
        <span className="flex-1">Data</span>
      </div>

      {/* Rows */}
      <div className="flex-1 overflow-auto">
        {displayFrames.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <p className="text-xs text-muted-foreground">No frames yet</p>
            <p className="text-[10px] text-muted-foreground">
              Connect to a CAN bus to start capturing
            </p>
          </div>
        ) : (
          <div className="divide-y divide-border/30">
            {displayFrames.map((frame, i) => {
              const typeInfo = getTypeLabel(frame.cob_id);
              return (
                <div
                  key={i}
                  className="flex items-center gap-2 px-3 py-0.5 text-[10px] font-mono hover:bg-muted/30 transition-colors"
                >
                  <span className="w-14 text-muted-foreground">
                    {formatTime(frame.timestamp_ms)}
                  </span>
                  <span className="w-16 font-medium">
                    0x{frame.cob_id.toString(16).padStart(3, '0').toUpperCase()}
                  </span>
                  <span className={`w-12 ${typeInfo.color}`}>{typeInfo.label}</span>
                  <span
                    className={`w-6 ${frame.direction === 'tx' ? 'text-blue-400' : 'text-green-400'}`}
                  >
                    {frame.direction.toUpperCase()}
                  </span>
                  <span className="w-6 text-muted-foreground">{frame.dlc}</span>
                  <span className="flex-1 truncate">{formatData(frame.data)}</span>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
