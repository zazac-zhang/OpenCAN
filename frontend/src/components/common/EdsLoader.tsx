/**
 * EdsLoader — A compact inline component for picking and loading
 * EDS (Electronic Data Sheet) files via the Tauri dialog plugin.
 *
 * Displays the selected file path, a loading indicator during parsing,
 * and EDS metadata (product name, vendor ID, product code) after
 * successful loading.
 */

import { open } from '@tauri-apps/plugin-dialog';
import { AlertCircle, FileCheck, FolderOpen, Loader2 } from 'lucide-react';
import { useState } from 'react';
import { useLoadEdsFile } from '@/hooks/useCommands';
import type { EdsInfo } from '@/lib/tauri';
import { cn } from '@/lib/utils';

export interface EdsLoaderProps {
  className?: string;
}

export function EdsLoader({ className }: EdsLoaderProps) {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [edsInfo, setEdsInfo] = useState<EdsInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadMutation = useLoadEdsFile();

  const handlePickFile = async () => {
    setError(null);
    setEdsInfo(null);

    const selected = await open({
      title: 'Select EDS File',
      filters: [{ name: 'EDS', extensions: ['eds'] }],
    });

    if (!selected || Array.isArray(selected)) return;

    setFilePath(selected);
    loadMutation.mutate(selected, {
      onSuccess: (info) => {
        setEdsInfo(info);
      },
      onError: (err) => {
        setError(String(err));
      },
    });
  };

  const isLoading = loadMutation.isPending;

  return (
    <div className={cn('flex flex-col gap-2', className)}>
      {/* File picker row */}
      <div className="flex items-center gap-2">
        <button
          className={cn(
            'inline-flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md border transition-colors',
            'hover:bg-muted disabled:opacity-50',
          )}
          onClick={handlePickFile}
          disabled={isLoading}
        >
          {isLoading ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
          ) : (
            <FolderOpen className="w-3.5 h-3.5" />
          )}
          {isLoading ? 'Loading...' : 'Open EDS...'}
        </button>

        {filePath && (
          <span
            className="text-xs font-mono text-muted-foreground truncate max-w-xs"
            title={filePath}
          >
            {filePath.split('/').pop()}
          </span>
        )}
      </div>

      {/* Loading state */}
      {isLoading && (
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Loader2 className="w-3 h-3 animate-spin" />
          Parsing EDS file...
        </div>
      )}

      {/* Error state */}
      {error && (
        <div className="flex items-center gap-1.5 text-xs text-destructive">
          <AlertCircle className="w-3.5 h-3.5" />
          {error}
        </div>
      )}

      {/* Success: EDS info */}
      {edsInfo && (
        <div className="flex items-center gap-1.5 px-3 py-2 rounded-md border bg-muted/50 text-xs">
          <FileCheck className="w-3.5 h-3.5 text-green-500 shrink-0" />
          <span className="font-medium">{edsInfo.product_name || 'Unnamed device'}</span>
          <span className="text-muted-foreground">
            (Vendor: 0x{edsInfo.vendor_id.toString(16).toUpperCase()}, Product: 0x
            {edsInfo.product_code.toString(16).toUpperCase()})
          </span>
        </div>
      )}
    </div>
  );
}
