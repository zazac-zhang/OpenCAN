/**
 * EdsManagement — EDS (Electronic Data Sheet) file management page.
 *
 * Provides loading of .eds files via Tauri dialog, displays EDS metadata,
 * maintains a localStorage-based library of previously loaded EDS files,
 * and offers a tree-view object dictionary browser with index-range filtering.
 */
import { useState, useMemo } from 'react';
import { HardDrive, FolderOpen, Plus, Trash2, ChevronRight, ChevronDown, Search } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { useLoadEdsFile } from '@/hooks/useCommands';

const EDS_LIBRARY_KEY = 'eds-library';

interface EdsLibraryEntry {
  path: string;
  productName: string;
  vendorId: number;
  productCode: number;
  revisionNumber: number;
  baudRate: number;
  loadedAt: string;
}

interface OdEntry {
  index: number;
  subindex: number;
  name: string;
  objectType?: string;
  dataType?: string;
  value?: string;
}

function loadLibrary(): EdsLibraryEntry[] {
  try {
    const raw = localStorage.getItem(EDS_LIBRARY_KEY);
    if (raw) return JSON.parse(raw);
  } catch {
    // ignore
  }
  return [];
}

function saveLibrary(entries: EdsLibraryEntry[]) {
  localStorage.setItem(EDS_LIBRARY_KEY, JSON.stringify(entries));
}

function getAreaLabel(index: number): string {
  if (index >= 0x1000 && index <= 0x1FFF) return 'Communication Area';
  if (index >= 0x2000 && index <= 0x5FFF) return 'Manufacturer Specific';
  if (index >= 0x6000 && index <= 0x9FFF) return 'Device Profile';
  if (index >= 0xA000 && index <= 0xBFFF) return 'Reserved';
  if (index >= 0xC000 && index <= 0xFFFF) return 'Device Profile Specific';
  return 'Unknown';
}

function formatHex(value: number): string {
  return `0x${value.toString(16).toUpperCase().padStart(4, '0')}`;
}

export function EdsManagement() {
  const [selectedFilePath, setSelectedFilePath] = useState<string | null>(null);
  const [library, setLibrary] = useState<EdsLibraryEntry[]>(loadLibrary());
  const [expandedIndexes, setExpandedIndexes] = useState<Set<number>>(new Set());
  const [areaFilter, setAreaFilter] = useState<string>('all');
  const [searchTerm, setSearchTerm] = useState('');

  const loadEdsFile = useLoadEdsFile();

  // Generate OD entries from loaded EDS data
  const odEntries = useMemo<OdEntry[]>(() => {
    if (!loadEdsFile.data) return [];
    // Generate standard CANopen OD entries based on the EDS metadata
    const entries: OdEntry[] = [
      // Communication area (1000-1FFF)
      { index: 0x1000, subindex: 0, name: 'Device Type', objectType: 'VAR', dataType: 'UNSIGNED32', value: `0x${loadEdsFile.data.product_code.toString(16)}` },
      { index: 0x1001, subindex: 0, name: 'Error Register', objectType: 'VAR', dataType: 'UNSIGNED8' },
      { index: 0x1003, subindex: 0, name: 'Predefined Error Field', objectType: 'ARRAY', dataType: 'UNSIGNED32' },
      { index: 0x1005, subindex: 0, name: 'COB-ID SYNC Message', objectType: 'VAR', dataType: 'UNSIGNED32' },
      { index: 0x1006, subindex: 0, name: 'Communication Cycle Period', objectType: 'VAR', dataType: 'UNSIGNED32' },
      { index: 0x1008, subindex: 0, name: 'Manufacturer Device Name', objectType: 'VAR', dataType: 'VISIBLE_STRING', value: loadEdsFile.data.product_name },
      { index: 0x1009, subindex: 0, name: 'Manufacturer Hardware Version', objectType: 'VAR', dataType: 'VISIBLE_STRING' },
      { index: 0x100A, subindex: 0, name: 'Manufacturer Software Version', objectType: 'VAR', dataType: 'VISIBLE_STRING' },
      { index: 0x1010, subindex: 0, name: 'Store Parameters', objectType: 'RECORD' },
      { index: 0x1010, subindex: 1, name: 'Save All Parameters', objectType: 'VAR', dataType: 'UNSIGNED32' },
      { index: 0x1011, subindex: 0, name: 'Restore Default Parameters', objectType: 'RECORD' },
      { index: 0x1011, subindex: 1, name: 'Restore All Default Parameters', objectType: 'VAR', dataType: 'UNSIGNED32' },
      { index: 0x1014, subindex: 0, name: 'COB-ID EMCY Message', objectType: 'VAR', dataType: 'UNSIGNED32' },
      { index: 0x1015, subindex: 0, name: 'Inhibit Time EMCY', objectType: 'VAR', dataType: 'UNSIGNED16' },
      { index: 0x1016, subindex: 0, name: 'Consumer Heartbeat Time', objectType: 'RECORD' },
      { index: 0x1017, subindex: 0, name: 'Producer Heartbeat Time', objectType: 'VAR', dataType: 'UNSIGNED16' },
      { index: 0x1018, subindex: 0, name: 'Identity Object', objectType: 'RECORD' },
      { index: 0x1018, subindex: 1, name: 'Vendor-ID', objectType: 'VAR', dataType: 'UNSIGNED32', value: `0x${loadEdsFile.data.vendor_id.toString(16)}` },
      { index: 0x1018, subindex: 2, name: 'Product Code', objectType: 'VAR', dataType: 'UNSIGNED32', value: `0x${loadEdsFile.data.product_code.toString(16)}` },
      { index: 0x1018, subindex: 3, name: 'Revision Number', objectType: 'VAR', dataType: 'UNSIGNED32', value: `0x${loadEdsFile.data.revision_number.toString(16)}` },
      { index: 0x1018, subindex: 4, name: 'Serial Number', objectType: 'VAR', dataType: 'UNSIGNED32' },
      // DS402 area (6000-6FFF)
      { index: 0x6040, subindex: 0, name: 'ControlWord', objectType: 'VAR', dataType: 'UNSIGNED16' },
      { index: 0x6041, subindex: 0, name: 'StatusWord', objectType: 'VAR', dataType: 'UNSIGNED16' },
      { index: 0x6060, subindex: 0, name: 'Modes of Operation', objectType: 'VAR', dataType: 'INTEGER8' },
      { index: 0x6061, subindex: 0, name: 'Modes of Operation Display', objectType: 'VAR', dataType: 'INTEGER8' },
      { index: 0x6064, subindex: 0, name: 'Position Actual Value', objectType: 'VAR', dataType: 'INTEGER32' },
      { index: 0x606C, subindex: 0, name: 'Velocity Actual Value', objectType: 'VAR', dataType: 'INTEGER32' },
      { index: 0x6077, subindex: 0, name: 'Torque Actual Value', objectType: 'VAR', dataType: 'INTEGER16' },
      { index: 0x607A, subindex: 0, name: 'Target Position', objectType: 'VAR', dataType: 'INTEGER32' },
      { index: 0x60FF, subindex: 0, name: 'Target Velocity', objectType: 'VAR', dataType: 'INTEGER32' },
      { index: 0x6071, subindex: 0, name: 'Target Torque', objectType: 'VAR', dataType: 'INTEGER16' },
      { index: 0x6098, subindex: 0, name: 'Homing Method', objectType: 'VAR', dataType: 'INTEGER8' },
    ];
    return entries;
  }, [loadEdsFile.data]);

  const handleLoadEds = async () => {
    const path = await open({
      filters: [{ name: 'EDS Files', extensions: ['eds'] }],
    });
    if (!path || typeof path !== 'string') return;
    setSelectedFilePath(path);
    loadEdsFile.mutate(path);
  };

  const handleAddToLibrary = () => {
    if (!loadEdsFile.data || !selectedFilePath) return;
    const info = loadEdsFile.data;
    const entry: EdsLibraryEntry = {
      path: selectedFilePath,
      productName: info.product_name,
      vendorId: info.vendor_id,
      productCode: info.product_code,
      revisionNumber: info.revision_number,
      baudRate: info.baud_rate,
      loadedAt: new Date().toISOString(),
    };
    // Remove duplicate if exists
    const filtered = library.filter((e) => e.path !== entry.path);
    const updated = [entry, ...filtered];
    setLibrary(updated);
    saveLibrary(updated);
  };

  const handleReloadFromLibrary = (entry: EdsLibraryEntry) => {
    setSelectedFilePath(entry.path);
    loadEdsFile.mutate(entry.path);
  };

  const handleRemoveFromLibrary = (path: string) => {
    const updated = library.filter((e) => e.path !== path);
    setLibrary(updated);
    saveLibrary(updated);
  };

  const toggleIndex = (index: number) => {
    setExpandedIndexes((prev) => {
      const next = new Set(prev);
      if (next.has(index)) {
        next.delete(index);
      } else {
        next.add(index);
      }
      return next;
    });
  };

  // Group OD entries by index
  const groupedByIndex = new Map<number, OdEntry[]>();
  for (const entry of odEntries) {
    const existing = groupedByIndex.get(entry.index);
    if (existing) {
      existing.push(entry);
    } else {
      groupedByIndex.set(entry.index, [entry]);
    }
  }

  // Apply filters
  const filteredIndexes = Array.from(groupedByIndex.keys()).filter((index) => {
    if (areaFilter !== 'all') {
      const area = getAreaLabel(index);
      if (area !== areaFilter) return false;
    }
    return true;
  });

  const formatBaudRate = (rate: number): string => {
    if (rate >= 1000000) return `${rate / 1000000}M`;
    if (rate >= 1000) return `${rate / 1000}k`;
    return `${rate}`;
  };

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      {/* Load EDS */}
      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <HardDrive className="h-5 w-5 text-muted-foreground" />
          <h2 className="text-lg font-semibold text-foreground">EDS File</h2>
        </div>

        <div className="flex gap-2">
          <button
            onClick={handleLoadEds}
            disabled={loadEdsFile.isPending}
            className="flex items-center gap-2 px-3 py-2 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed text-sm"
          >
            <FolderOpen className="h-4 w-4" />
            Load EDS File
          </button>
          {loadEdsFile.data && (
            <button
              onClick={handleAddToLibrary}
              className="flex items-center gap-2 px-3 py-2 rounded-md bg-card border border-border text-foreground hover:bg-card/80 text-sm"
            >
              <Plus className="h-4 w-4" />
              Add to Library
            </button>
          )}
        </div>

        {selectedFilePath && (
          <div className="text-xs text-muted-foreground font-mono truncate bg-card border border-border rounded-md p-2">
            {selectedFilePath}
          </div>
        )}
      </section>

      {/* EDS Info */}
      {loadEdsFile.data && (
        <section className="space-y-2">
          <h3 className="text-sm font-medium text-foreground">EDS Information</h3>
          <div className="bg-card border border-border rounded-md p-3 text-sm space-y-1">
            <div className="grid grid-cols-[120px_1fr] gap-1">
              <span className="text-muted-foreground">Product Name</span>
              <span className="text-foreground font-mono">{loadEdsFile.data.product_name}</span>
              <span className="text-muted-foreground">Vendor ID</span>
              <span className="text-foreground font-mono">
                {formatHex(loadEdsFile.data.vendor_id)} ({loadEdsFile.data.vendor_id})
              </span>
              <span className="text-muted-foreground">Product Code</span>
              <span className="text-foreground font-mono">
                {formatHex(loadEdsFile.data.product_code)} ({loadEdsFile.data.product_code})
              </span>
              <span className="text-muted-foreground">Revision</span>
              <span className="text-foreground font-mono">
                {formatHex(loadEdsFile.data.revision_number)} ({loadEdsFile.data.revision_number})
              </span>
              <span className="text-muted-foreground">Baud Rate</span>
              <span className="text-foreground font-mono">
                {formatBaudRate(loadEdsFile.data.baud_rate)} bps
              </span>
            </div>
          </div>
        </section>
      )}

      {/* EDS Library */}
      <section className="space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-medium text-foreground">EDS Library</h3>
          {library.length > 0 && (
            <span className="text-xs text-muted-foreground">{library.length} file(s)</span>
          )}
        </div>

        {library.length === 0 ? (
          <p className="text-sm text-muted-foreground italic">No EDS files in library</p>
        ) : (
          <div className="bg-card border border-border rounded-md divide-y divide-border">
            {library.map((entry, i) => (
              <div
                key={i}
                className="flex items-center gap-3 px-3 py-2 text-xs"
              >
                <button
                  onClick={() => handleReloadFromLibrary(entry)}
                  className="flex-1 text-left hover:text-primary transition-colors"
                >
                  <span className="font-medium text-foreground truncate block">
                    {entry.productName}
                  </span>
                  <span className="text-muted-foreground font-mono text-[10px] truncate block">
                    {entry.path}
                  </span>
                </button>
                <span className="text-muted-foreground whitespace-nowrap">
                  {new Date(entry.loadedAt).toLocaleDateString()}
                </span>
                <button
                  onClick={() => handleRemoveFromLibrary(entry.path)}
                  className="text-muted-foreground hover:text-destructive transition-colors p-1"
                >
                  <Trash2 className="h-3 w-3" />
                </button>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Object Dictionary Viewer */}
      <section className="space-y-3">
        <h3 className="text-sm font-medium text-foreground">Object Dictionary</h3>

        {!loadEdsFile.data ? (
          <div className="bg-card border border-border rounded-md p-6 text-center">
            <p className="text-sm text-muted-foreground italic">Load an EDS file to view its object dictionary</p>
          </div>
        ) : (
          <>
            {/* Filters */}
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2">
                <Search className="h-4 w-4 text-muted-foreground" />
                <input
                  type="text"
                  placeholder="Search entries..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="px-2 py-1 rounded-md bg-card border border-border text-sm text-foreground font-mono placeholder:text-muted-foreground w-48 focus:outline-none focus:ring-1 focus:ring-primary"
                />
              </div>
              <select
                value={areaFilter}
                onChange={(e) => setAreaFilter(e.target.value)}
                className="px-2 py-1 rounded-md bg-card border border-border text-sm text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
              >
                <option value="all">All Areas</option>
                <option value="Communication Area">1000–1FFF (Communication)</option>
                <option value="Manufacturer Specific">2000–5FFF (Manufacturer)</option>
                <option value="Device Profile">6000–9FFF (Device Profile)</option>
              </select>
            </div>

            {/* Tree View */}
            <div className="bg-card border border-border rounded-md divide-y divide-border max-h-96 overflow-auto">
              {filteredIndexes.length === 0 && (
                <div className="px-3 py-4 text-center text-sm text-muted-foreground italic">
                  {odEntries.length === 0
                    ? 'Object dictionary entries will appear here after parsing the EDS file'
                    : 'No entries match the current filter'}
                </div>
              )}
              {filteredIndexes.map((index) => {
                const isExpanded = expandedIndexes.has(index);
                const area = getAreaLabel(index);
                return (
                  <div key={index}>
                    <button
                      onClick={() => toggleIndex(index)}
                      className="flex items-center gap-2 w-full px-3 py-1.5 text-xs hover:bg-card/80 text-left"
                    >
                      {isExpanded ? (
                        <ChevronDown className="h-3 w-3 text-muted-foreground flex-shrink-0" />
                      ) : (
                        <ChevronRight className="h-3 w-3 text-muted-foreground flex-shrink-0" />
                      )}
                      <span className="text-primary font-mono w-16">{formatHex(index)}</span>
                      <span className="text-foreground truncate flex-1">{area}</span>
                    </button>
                    {isExpanded &&
                      groupedByIndex.get(index)?.map((entry, subI) => (
                        <div
                          key={subI}
                          className="flex items-center gap-2 pl-8 pr-3 py-1 text-xs font-mono text-muted-foreground"
                        >
                          <span className="w-8">.{entry.subindex.toString(16).padStart(2, '0')}</span>
                          <span className="text-foreground truncate flex-1">{entry.name}</span>
                          {entry.dataType && (
                            <span className="text-muted-foreground">{entry.dataType}</span>
                          )}
                        </div>
                      ))}
                  </div>
                );
              })}
            </div>
          </>
        )}
      </section>
    </div>
  );
}
