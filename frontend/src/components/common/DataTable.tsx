/**
 * DataTable — A reusable virtualized table component built with
 * `@tanstack/react-table` and `@tanstack/react-virtual`.
 *
 * Supports generic row data, auto-reverse toggle, sticky header,
 * monospace data columns, and a footer with row count.
 */

import { type ColumnDef, getCoreRowModel, type Row, useReactTable } from '@tanstack/react-table';
import { useVirtualizer } from '@tanstack/react-virtual';
import { ArrowDownUp } from 'lucide-react';
import { useMemo, useRef, useState } from 'react';
import { cn } from '@/lib/utils';

export interface DataTableProps<T> {
  columns: ColumnDef<T, unknown>[];
  data: T[];
  maxRows?: number;
  rowHeight?: number;
  className?: string;
}

export function DataTable<T>({
  columns,
  data,
  maxRows = 10000,
  rowHeight = 24,
  className,
}: DataTableProps<T>) {
  const [reversed, setReversed] = useState(false);

  const limitedData = useMemo(() => data.slice(-maxRows), [data, maxRows]);

  const tableData = useMemo(
    () => (reversed ? [...limitedData].reverse() : limitedData),
    [limitedData, reversed],
  );

  const table = useReactTable({
    data: tableData,
    columns,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true,
  });

  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: table.getRowModel().rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => rowHeight,
    overscan: 5,
  });

  return (
    <div className={cn('flex flex-col h-full', className)}>
      {/* Header bar with reverse toggle */}
      <div className="flex items-center justify-between px-3 py-1 border-b bg-background shrink-0">
        <button
          className={cn(
            'flex items-center gap-1 text-xs px-2 py-1 rounded border transition-colors',
            reversed
              ? 'bg-primary text-primary-foreground border-primary'
              : 'hover:bg-muted text-muted-foreground',
          )}
          onClick={() => setReversed((r) => !r)}
          title="Toggle newest-first order"
        >
          <ArrowDownUp className="w-3 h-3" />
          {reversed ? 'Newest first' : 'Oldest first'}
        </button>
        <span className="text-xs text-muted-foreground">
          {table.getRowModel().rows.length} rows
        </span>
      </div>

      {/* Table header (sticky) */}
      <div className="flex border-b bg-muted/50 text-xs font-medium shrink-0">
        {table.getFlatHeaders().map((header) => {
          const width = (header.column.columnDef.meta as { width?: string })?.width ?? 'auto';
          return (
            <div
              key={header.id}
              className="px-3 py-1 truncate"
              style={{ width, minWidth: width === 'auto' ? undefined : width }}
            >
              {(header.column.columnDef.header as string) ?? header.id}
            </div>
          );
        })}
      </div>

      {/* Virtualized body */}
      <div ref={parentRef} className="flex-1 overflow-auto">
        <div style={{ height: `${virtualizer.getTotalSize()}px`, position: 'relative' }}>
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const row = table.getRowModel().rows[virtualRow.index] as Row<T> | undefined;
            if (!row) return null;

            return (
              <div
                key={virtualRow.index}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  height: `${virtualRow.size}px`,
                  transform: `translateY(${virtualRow.start}px)`,
                }}
                className="flex items-center text-xs border-b hover:bg-muted/50"
              >
                {row.getVisibleCells().map((cell) => {
                  const width = (cell.column.columnDef.meta as { width?: string })?.width ?? 'auto';
                  const isDataCol = cell.column.id !== '__index__';
                  return (
                    <div
                      key={cell.id}
                      className={cn('px-3 truncate', isDataCol && 'font-mono')}
                      style={{ width, minWidth: width === 'auto' ? undefined : width }}
                    >
                      {cell.column.columnDef.cell
                        ? (
                            cell.column.columnDef.cell as (info: {
                              getValue: () => unknown;
                            }) => React.ReactNode
                          )({ getValue: cell.getValue })
                        : String(cell.getValue() ?? '')}
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>

        {table.getRowModel().rows.length === 0 && (
          <div className="flex items-center justify-center h-full text-sm text-muted-foreground">
            No data available
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="px-3 py-1 text-xs text-muted-foreground border-t shrink-0">
        {limitedData.length} row{limitedData.length !== 1 ? 's' : ''}
        {limitedData.length >= maxRows && ` (capped at ${maxRows.toLocaleString()})`}
      </div>
    </div>
  );
}
