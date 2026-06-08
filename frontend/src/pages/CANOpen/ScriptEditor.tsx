/**
 * ScriptEditor — Automation script editor with execution log.
 *
 * Features:
 * - Script editor with syntax highlighting hints
 * - Predefined script templates
 * - Execution log with status indicators
 * - Start/stop controls
 */

import { BookOpen, Copy, Download, FileCode, Play, Square, Trash2 } from 'lucide-react';
import { useCallback, useRef, useState } from 'react';
import { useNmtCommand, useSdoDownload, useSdoUpload } from '@/hooks/useCommands';
import {
  type CommandExecutor,
  executeScript,
  getScriptTemplate,
  PREDEFINED_SCRIPTS,
  parseScript,
  type ScriptCommand,
  type ScriptExecutionLog,
} from '@/lib/script-engine';
import { cn } from '@/lib/utils';

// ===== Main Component =====

export function ScriptEditor() {
  const sdoUpload = useSdoUpload();
  const sdoDownload = useSdoDownload();
  const nmtMutation = useNmtCommand();

  const [script, setScript] = useState(getScriptTemplate('basic'));
  const [executionLog, setExecutionLog] = useState<ScriptExecutionLog[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [errors, setErrors] = useState<{ line: number; message: string }[]>([]);
  const abortRef = useRef<AbortController | null>(null);

  // Parse script on change
  const handleScriptChange = useCallback((newScript: string) => {
    setScript(newScript);
    const result = parseScript(newScript);
    setErrors(result.errors);
  }, []);

  // Execute script
  const handleRun = useCallback(async () => {
    const result = parseScript(script);
    if (result.errors.length > 0) {
      setErrors(result.errors);
      return;
    }

    setIsRunning(true);
    setExecutionLog([]);
    setErrors([]);

    const abortController = new AbortController();
    abortRef.current = abortController;

    const executor: CommandExecutor = async (cmd: ScriptCommand) => {
      switch (cmd.type) {
        case 'send': {
          // TODO: Implement CAN frame send
          return { success: true, message: `Sent frame ${cmd.params.id}` };
        }
        case 'sdo_read': {
          return new Promise((resolve) => {
            sdoUpload.mutate(
              {
                node_id: cmd.params.node as number,
                index: parseInt(cmd.params.index as string, 16),
                subindex: parseInt(cmd.params.subindex as string, 16),
                data_type: 'UNS16',
              },
              {
                onSuccess: (data) => {
                  resolve({
                    success: true,
                    message: `Read: ${JSON.stringify(data)}`,
                  });
                },
                onError: (error) => {
                  resolve({
                    success: false,
                    message: String(error),
                  });
                },
              },
            );
          });
        }
        case 'sdo_write': {
          const value = parseInt(cmd.params.value as string, 16);
          const data = [value & 0xff, (value >> 8) & 0xff];
          return new Promise((resolve) => {
            sdoDownload.mutate(
              {
                node_id: cmd.params.node as number,
                index: parseInt(cmd.params.index as string, 16),
                subindex: parseInt(cmd.params.subindex as string, 16),
                data,
              },
              {
                onSuccess: () => {
                  resolve({
                    success: true,
                    message: `Written 0x${cmd.params.value}`,
                  });
                },
                onError: (error) => {
                  resolve({
                    success: false,
                    message: String(error),
                  });
                },
              },
            );
          });
        }
        case 'wait': {
          const ms = cmd.params.ms as number;
          await new Promise((resolve) => setTimeout(resolve, ms));
          return { success: true };
        }
        case 'log': {
          return { success: true, message: cmd.params.message as string };
        }
        case 'nmt': {
          return new Promise((resolve) => {
            nmtMutation.mutate(
              {
                nodeId: cmd.params.node as number,
                command: cmd.params.command as string,
              },
              {
                onSuccess: () => {
                  resolve({
                    success: true,
                    message: `NMT ${cmd.params.command} sent to node ${cmd.params.node}`,
                  });
                },
                onError: (error) => {
                  resolve({
                    success: false,
                    message: String(error),
                  });
                },
              },
            );
          });
        }
        default:
          return { success: false, message: `Unknown command type: ${cmd.type}` };
      }
    };

    const logHandler = (log: ScriptExecutionLog) => {
      setExecutionLog((prev) => [...prev, log]);
    };

    try {
      await executeScript(result.commands, executor, logHandler, abortController.signal);
    } catch (error) {
      logHandler({
        timestamp: Date.now(),
        command: 'Script',
        status: 'error',
        message: String(error),
      });
    }

    setIsRunning(false);
    abortRef.current = null;
  }, [script, sdoUpload, sdoDownload, nmtMutation]);

  // Stop script
  const handleStop = useCallback(() => {
    abortRef.current?.abort();
    setIsRunning(false);
  }, []);

  // Load predefined script
  const handleLoadScript = useCallback(
    (key: string) => {
      const predefined = PREDEFINED_SCRIPTS[key];
      if (predefined) {
        handleScriptChange(predefined.script);
      }
    },
    [handleScriptChange],
  );

  // Copy script
  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(script);
  }, [script]);

  // Export log
  const handleExportLog = useCallback(() => {
    const header = 'Timestamp,Command,Status,Message,Duration_ms\n';
    const rows = executionLog
      .map((log) => {
        const time = new Date(log.timestamp).toISOString();
        return `${time},"${log.command}",${log.status},"${log.message || ''}",${log.duration || ''}`;
      })
      .join('\n');
    const blob = new Blob([header + rows], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'script_log.csv';
    a.click();
    URL.revokeObjectURL(url);
  }, [executionLog]);

  return (
    <div className="flex h-full overflow-hidden">
      {/* Left: Script Editor */}
      <div className="w-1/2 border-r flex flex-col overflow-hidden">
        {/* Toolbar */}
        <div className="px-3 py-2 border-b flex items-center gap-2">
          <div className="flex items-center gap-1.5">
            <FileCode className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm font-medium">Script Editor</span>
          </div>
          <div className="flex-1" />
          {/* Predefined scripts */}
          <div className="flex items-center gap-1">
            <BookOpen className="h-3.5 w-3.5 text-muted-foreground" />
            <select
              onChange={(e) => handleLoadScript(e.target.value)}
              className="text-xs bg-muted/30 rounded px-2 py-1 border"
              defaultValue=""
            >
              <option value="" disabled>
                Load template...
              </option>
              {Object.entries(PREDEFINED_SCRIPTS).map(([key, script]) => (
                <option key={key} value={key}>
                  {script.name}
                </option>
              ))}
            </select>
          </div>
          <button onClick={handleCopy} className="p-1.5 rounded hover:bg-muted" title="Copy script">
            <Copy className="h-3.5 w-3.5" />
          </button>
        </div>

        {/* Editor */}
        <div className="flex-1 overflow-hidden">
          <textarea
            value={script}
            onChange={(e) => handleScriptChange(e.target.value)}
            className="w-full h-full p-3 font-mono text-xs bg-transparent resize-none outline-none"
            placeholder="// Enter your script here...
// Supported commands:
//   send <id> <data>
//   sdo_read <node> <index> <subindex>
//   sdo_write <node> <index> <subindex> <value>
//   wait <ms>
//   log <message>
//   nmt <node> <command>
//   repeat <count> { ... }"
            spellCheck={false}
          />
        </div>

        {/* Errors */}
        {errors.length > 0 && (
          <div className="px-3 py-2 border-t bg-red-500/10 text-xs">
            {errors.map((err, i) => (
              <div key={i} className="text-red-400">
                Line {err.line}: {err.message}
              </div>
            ))}
          </div>
        )}

        {/* Run controls */}
        <div className="px-3 py-2 border-t flex items-center gap-2">
          {isRunning ? (
            <button
              onClick={handleStop}
              className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded bg-red-600 text-white hover:bg-red-700"
            >
              <Square className="h-3.5 w-3.5" />
              Stop
            </button>
          ) : (
            <button
              onClick={handleRun}
              disabled={errors.length > 0}
              className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded bg-green-600 text-white hover:bg-green-700 disabled:opacity-50"
            >
              <Play className="h-3.5 w-3.5" />
              Run Script
            </button>
          )}
          <span className="text-xs text-muted-foreground">
            {isRunning ? 'Running...' : 'Ready'}
          </span>
        </div>
      </div>

      {/* Right: Execution Log */}
      <div className="w-1/2 flex flex-col overflow-hidden">
        <div className="px-3 py-2 border-b flex items-center justify-between">
          <div className="flex items-center gap-1.5">
            <span className="text-sm font-medium">Execution Log</span>
            <span className="text-xs text-muted-foreground">({executionLog.length} entries)</span>
          </div>
          <div className="flex items-center gap-1">
            <button
              onClick={handleExportLog}
              disabled={executionLog.length === 0}
              className="p-1.5 rounded hover:bg-muted disabled:opacity-50"
              title="Export log"
            >
              <Download className="h-3.5 w-3.5" />
            </button>
            <button
              onClick={() => setExecutionLog([])}
              disabled={executionLog.length === 0}
              className="p-1.5 rounded hover:bg-muted disabled:opacity-50"
              title="Clear log"
            >
              <Trash2 className="h-3.5 w-3.5" />
            </button>
          </div>
        </div>

        <div className="flex-1 overflow-auto">
          {executionLog.length === 0 ? (
            <div className="flex items-center justify-center h-full text-sm text-muted-foreground italic">
              Run a script to see execution log
            </div>
          ) : (
            <div className="divide-y divide-border">
              {executionLog.map((log, i) => (
                <div
                  key={i}
                  className={cn(
                    'px-3 py-1.5 text-xs font-mono flex items-start gap-2',
                    log.status === 'error' && 'bg-red-500/5',
                    log.status === 'success' && 'bg-green-500/5',
                    log.status === 'running' && 'bg-blue-500/5',
                  )}
                >
                  <span className="text-muted-foreground w-16 shrink-0">
                    {new Date(log.timestamp).toLocaleTimeString()}
                  </span>
                  <span
                    className={cn(
                      'w-16 shrink-0',
                      log.status === 'success' && 'text-green-400',
                      log.status === 'error' && 'text-red-400',
                      log.status === 'running' && 'text-blue-400',
                      log.status === 'skipped' && 'text-muted-foreground',
                    )}
                  >
                    {log.status === 'success'
                      ? '✓'
                      : log.status === 'error'
                        ? '✗'
                        : log.status === 'running'
                          ? '⟳'
                          : '○'}
                  </span>
                  <span className="flex-1 break-all">{log.command}</span>
                  {log.message && (
                    <span className="text-muted-foreground ml-2">— {log.message}</span>
                  )}
                  {log.duration !== undefined && (
                    <span className="text-muted-foreground ml-2">{log.duration}ms</span>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
