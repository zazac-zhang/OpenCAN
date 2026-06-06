/**
 * ControlPanel — DS402 motion control panel with target value inputs.
 *
 * Dynamically shows input fields based on the current DS402 operation mode:
 * - PP/CSP (1/8): Position + Velocity profile inputs
 * - PV/CSV (3/9): Velocity input
 * - CST (10): Torque input
 * - Homing (6): Homing method selector + Start button
 *
 * Each input has a "Set" button that dispatches the `useDs402SetTarget` mutation.
 * Actual (read-only) position/velocity/torque values are displayed when provided.
 */

import { useState } from 'react';
import { Settings, ArrowRight, Gauge, Zap, Home, RotateCcw } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useDs402SetTarget } from '@/hooks/useCommands';

export interface ControlPanelProps {
  /** CANopen node ID */
  nodeId: number;
  /** Current DS402 mode name (e.g. "PP", "CSP", "PV") */
  currentMode: string;
  /** Target position value (controlled input) */
  targetPosition: string;
  /** Target velocity value (controlled input) */
  targetVelocity: string;
  /** Target torque value (controlled input) */
  targetTorque: string;
  /** Called when target position input changes */
  onTargetPositionChange: (value: string) => void;
  /** Called when target velocity input changes */
  onTargetVelocityChange: (value: string) => void;
  /** Called when target torque input changes */
  onTargetTorqueChange: (value: string) => void;
  /** Called when a target is set via the Set button */
  onSetTarget: (mode: number, value: number) => void;
  /** Optional actual position value for display */
  actualPosition?: number;
  /** Optional actual velocity value for display */
  actualVelocity?: number;
  /** Optional actual torque value for display */
  actualTorque?: number;
}

/** Homing methods per CiA 402 */
const HOMING_METHODS = [
  { value: 0, label: '0: Abort' },
  { value: 1, label: '1: Neg limit switch' },
  { value: 2, label: '2: Pos limit switch' },
  { value: 3, label: '3: Pos home switch' },
  { value: 4, label: '4: Neg home switch' },
  { value: 5, label: '5: Neg home + neg limit' },
  { value: 6, label: '6: Pos home + pos limit' },
  { value: 7, label: '7: Neg home + idx' },
  { value: 8, label: '8: Pos home + idx' },
  { value: 9, label: '9: Neg limit + idx' },
  { value: 10, label: '10: Pos limit + idx' },
  { value: 11, label: '11: Neg home + idx (vel search)' },
  { value: 12, label: '12: Pos home + idx (vel search)' },
  { value: 13, label: '13: Neg home + idx (dir change)' },
  { value: 14, label: '14: Pos home + idx (dir change)' },
  { value: 34, label: '34: No homing (current pos)' },
] as const;

/** Determine which input fields to show based on mode */
function getInputConfig(mode: string): {
  position: boolean;
  velocity: boolean;
  torque: boolean;
  homing: boolean;
} {
  switch (mode.toUpperCase()) {
    case 'PP':
    case 'CSP':
    case '1':
    case '8':
      return { position: true, velocity: true, torque: false, homing: false };
    case 'PV':
    case 'CSV':
    case '3':
    case '9':
    case '2': // Velocity mode
      return { position: false, velocity: true, torque: false, homing: false };
    case 'CST':
    case '10':
      return { position: false, velocity: false, torque: true, homing: false };
    case 'HM':
    case 'HOMING':
    case '6':
      return { position: false, velocity: false, torque: false, homing: true };
    default:
      return { position: true, velocity: true, torque: false, homing: false };
  }
}

/** Get the icon for the current mode */
function getModeIcon(mode: string) {
  switch (mode.toUpperCase()) {
    case 'PP':
    case 'CSP':
    case '1':
    case '8':
      return ArrowRight;
    case 'PV':
    case 'CSV':
    case '3':
    case '9':
    case 'VL':
    case '2':
      return Gauge;
    case 'CST':
    case '10':
      return Zap;
    case 'HM':
    case 'HOMING':
    case '6':
      return Home;
    default:
      return Settings;
  }
}

export function ControlPanel({
  nodeId,
  currentMode,
  targetPosition,
  targetVelocity,
  targetTorque,
  onTargetPositionChange,
  onTargetVelocityChange,
  onTargetTorqueChange,
  onSetTarget,
  actualPosition,
  actualVelocity,
  actualTorque,
}: ControlPanelProps) {
  const [homingMethod, setHomingMethod] = useState(1);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const setTargetMutation = useDs402SetTarget();

  const inputs = getInputConfig(currentMode);
  const ModeIcon = getModeIcon(currentMode);

  const modeNumber = (() => {
    switch (currentMode.toUpperCase()) {
      case 'PP': return 1;
      case 'VL': return 2;
      case 'PV': return 3;
      case 'HM':
      case 'HOMING': return 6;
      case 'IP': return 7;
      case 'CSP': return 8;
      case 'CSV': return 9;
      case 'CST': return 10;
      default: return parseInt(currentMode, 10) || 1;
    }
  })();

  const handleSetPosition = () => {
    const val = parseFloat(targetPosition);
    if (!isNaN(val)) {
      setTargetMutation.mutate({ nodeId, mode: modeNumber, target: val });
      onSetTarget(modeNumber, val);
    }
  };

  const handleSetVelocity = () => {
    const val = parseFloat(targetVelocity);
    if (!isNaN(val)) {
      setTargetMutation.mutate({ nodeId, mode: modeNumber, target: val });
      onSetTarget(modeNumber, val);
    }
  };

  const handleSetTorque = () => {
    const val = parseFloat(targetTorque);
    if (!isNaN(val)) {
      setTargetMutation.mutate({ nodeId, mode: modeNumber, target: val });
      onSetTarget(modeNumber, val);
    }
  };

  const handleStartHoming = () => {
    // Write homing method to 0x6098:00, then trigger homing via controlword
    setTargetMutation.mutate({ nodeId, mode: 6, target: homingMethod });
  };

  /** Reusable input row with label, input, Set button, and optional actual value */
  const InputRow = ({
    label,
    value,
    onChange,
    onSet,
    unit,
    actualValue,
    icon: RowIcon,
  }: {
    label: string;
    value: string;
    onChange: (v: string) => void;
    onSet: () => void;
    unit: string;
    actualValue?: number;
    icon: typeof ArrowRight;
  }) => (
    <div className="space-y-1">
      <div className="flex items-center gap-1.5">
        <RowIcon className="w-3 h-3 text-muted-foreground" />
        <span className="text-xs font-medium">{label}</span>
        {actualValue !== undefined && (
          <span className="ml-auto text-[11px] text-muted-foreground">
            Actual: {actualValue.toFixed(2)} {unit}
          </span>
        )}
      </div>
      <div className="flex items-center gap-1.5">
        <input
          type="text"
          inputMode="decimal"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={`Enter ${label.toLowerCase()}...`}
          className={cn(
            'flex-1 px-2 py-1.5 text-xs border rounded bg-background',
            'focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary',
          )}
        />
        <button
          onClick={onSet}
          disabled={setTargetMutation.isPending}
          className={cn(
            'px-3 py-1.5 text-xs font-medium rounded border transition-colors',
            setTargetMutation.isPending
              ? 'opacity-50 cursor-not-allowed'
              : 'bg-primary text-primary-foreground hover:bg-primary/90 border-primary/50',
          )}
        >
          Set
        </button>
      </div>
    </div>
  );

  return (
    <div className="space-y-3">
      {/* Header */}
      <div className="flex items-center gap-1.5">
        <ModeIcon className="w-3.5 h-3.5 text-muted-foreground" />
        <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Motion Control
        </span>
        <span className="ml-auto text-[10px] text-muted-foreground">
          Node {nodeId}
        </span>
      </div>

      {/* Auto-refresh toggle */}
      <label className="flex items-center gap-1.5 text-xs cursor-pointer">
        <input
          type="checkbox"
          checked={autoRefresh}
          onChange={(e) => setAutoRefresh(e.target.checked)}
          className="rounded border-border"
        />
        <span className="text-muted-foreground">Auto-refresh</span>
      </label>

      {/* Position input (PP, CSP) */}
      {inputs.position && (
        <InputRow
          label="Target Position"
          value={targetPosition}
          onChange={onTargetPositionChange}
          onSet={handleSetPosition}
          unit="pulses"
          actualValue={actualPosition}
          icon={ArrowRight}
        />
      )}

      {/* Velocity input (PP, CSP, PV, CSV) */}
      {inputs.velocity && (
        <InputRow
          label="Profile Velocity"
          value={targetVelocity}
          onChange={onTargetVelocityChange}
          onSet={handleSetVelocity}
          unit="rpm"
          actualValue={actualVelocity}
          icon={Gauge}
        />
      )}

      {/* Torque input (CST) */}
      {inputs.torque && (
        <InputRow
          label="Target Torque"
          value={targetTorque}
          onChange={onTargetTorqueChange}
          onSet={handleSetTorque}
          unit="%"
          actualValue={actualTorque}
          icon={Zap}
        />
      )}

      {/* Homing controls */}
      {inputs.homing && (
        <div className="space-y-2">
          <div className="flex items-center gap-1.5">
            <Home className="w-3 h-3 text-muted-foreground" />
            <span className="text-xs font-medium">Homing</span>
          </div>

          <div className="flex items-center gap-1.5">
            <select
              value={homingMethod}
              onChange={(e) => setHomingMethod(parseInt(e.target.value, 10))}
              className={cn(
                'flex-1 px-2 py-1.5 text-xs border rounded bg-background',
                'focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary',
              )}
            >
              {HOMING_METHODS.map((m) => (
                <option key={m.value} value={m.value}>
                  {m.label}
                </option>
              ))}
            </select>
            <button
              onClick={handleStartHoming}
              disabled={setTargetMutation.isPending}
              className={cn(
                'px-3 py-1.5 text-xs font-medium rounded border transition-colors',
                setTargetMutation.isPending
                  ? 'opacity-50 cursor-not-allowed'
                  : 'bg-primary text-primary-foreground hover:bg-primary/90 border-primary/50',
              )}
            >
              <RotateCcw className="w-3 h-3 mr-1 inline" />
              Start
            </button>
          </div>
        </div>
      )}

      {/* Mutation status */}
      {setTargetMutation.isError && (
        <div className="text-[11px] text-red-400">
          Error: {setTargetMutation.error?.message ?? 'Unknown error'}
        </div>
      )}
    </div>
  );
}
