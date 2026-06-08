/**
 * WaveformDisplay — Real-time waveform display for DS402 telemetry data.
 *
 * Renders three stacked charts for Position, Velocity, and Torque using
 * the shared Waveform component (`@/components/common/Waveform`).
 *
 * Features:
 * - Show/hide toggle checkboxes for each axis
 * - Current value display with units
 * - Scrollable container for limited space
 * - "No data" message when all histories are empty
 */

import { Activity, Eye, EyeOff } from 'lucide-react';
import { useState } from 'react';
import { Waveform } from '@/components/common/Waveform';
import type { DataPoint } from '@/types/ds402';

export interface WaveformDisplayProps {
  /** CANopen node ID */
  nodeId: number;
  /** Position time-series data */
  positionHistory: DataPoint[];
  /** Velocity time-series data */
  velocityHistory: DataPoint[];
  /** Torque time-series data */
  torqueHistory: DataPoint[];
}

export function WaveformDisplay({
  nodeId,
  positionHistory,
  velocityHistory,
  torqueHistory,
}: WaveformDisplayProps) {
  const [showPosition, setShowPosition] = useState(true);
  const [showVelocity, setShowVelocity] = useState(true);
  const [showTorque, setShowTorque] = useState(true);

  const hasData =
    positionHistory.length > 0 || velocityHistory.length > 0 || torqueHistory.length > 0;

  /** Toggle button for axis visibility */
  const ToggleBtn = ({
    visible,
    onToggle,
    label,
  }: {
    visible: boolean;
    onToggle: () => void;
    label: string;
  }) => (
    <button
      onClick={onToggle}
      className="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] text-muted-foreground hover:text-foreground transition-colors"
      title={visible ? `Hide ${label}` : `Show ${label}`}
    >
      {visible ? <Eye className="w-3 h-3" /> : <EyeOff className="w-3 h-3" />}
      <span>{label}</span>
    </button>
  );

  if (!hasData) {
    return (
      <div className="space-y-2">
        <div className="flex items-center gap-1.5">
          <Activity className="w-3.5 h-3.5 text-muted-foreground" />
          <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Telemetry
          </span>
          <span className="ml-auto text-[10px] text-muted-foreground">Node {nodeId}</span>
        </div>
        <div className="flex flex-col items-center justify-center border rounded bg-card py-8">
          <Activity className="w-6 h-6 text-muted-foreground/50 mb-2" />
          <span className="text-xs text-muted-foreground">No telemetry data for Node {nodeId}</span>
          <span className="text-[10px] text-muted-foreground/70 mt-1">
            Enable the drive and start motion to see waveforms
          </span>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-3 overflow-y-auto max-h-[500px]">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5">
          <Activity className="w-3.5 h-3.5 text-muted-foreground" />
          <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Telemetry
          </span>
        </div>
        <div className="flex items-center gap-1">
          <ToggleBtn
            visible={showPosition}
            onToggle={() => setShowPosition(!showPosition)}
            label="Position"
          />
          <ToggleBtn
            visible={showVelocity}
            onToggle={() => setShowVelocity(!showVelocity)}
            label="Velocity"
          />
          <ToggleBtn
            visible={showTorque}
            onToggle={() => setShowTorque(!showTorque)}
            label="Torque"
          />
        </div>
      </div>

      {/* Position chart */}
      {showPosition && (
        <Waveform
          data={positionHistory}
          label="Position"
          color="#3b82f6"
          height={120}
          className="shrink-0"
        />
      )}

      {/* Velocity chart */}
      {showVelocity && (
        <Waveform
          data={velocityHistory}
          label="Velocity"
          color="#22c55e"
          height={120}
          className="shrink-0"
        />
      )}

      {/* Torque chart */}
      {showTorque && (
        <Waveform
          data={torqueHistory}
          label="Torque"
          color="#f59e0b"
          height={120}
          className="shrink-0"
        />
      )}
    </div>
  );
}
