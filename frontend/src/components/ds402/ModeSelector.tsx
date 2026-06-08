/**
 * ModeSelector — DS402 operation mode selector.
 *
 * Displays the 8 standard CiA 402 operation modes as a grid of selectable
 * buttons (4 per row). Each button shows an icon and mode abbreviation.
 * The selected mode is highlighted with the primary color.
 */

import { ArrowRight, CircleDot, Gauge, Grid3X3, Home, Zap } from 'lucide-react';
import { cn } from '@/lib/utils';

export interface ModeSelectorProps {
  /** Currently selected mode number (1-10) */
  selectedMode: number;
  /** Callback when a mode is selected, receives the mode number */
  onModeChange: (mode: number) => void;
  /** Disables all mode buttons */
  disabled?: boolean;
}

/** DS402 operation modes with display info */
const MODES = [
  { value: 1, abbr: 'PP', name: 'Profile Position', icon: ArrowRight },
  { value: 2, abbr: 'VL', name: 'Velocity', icon: Gauge },
  { value: 3, abbr: 'PV', name: 'Profile Velocity', icon: Gauge },
  { value: 6, abbr: 'HM', name: 'Homing', icon: Home },
  { value: 7, abbr: 'IP', name: 'Interpolated Position', icon: Grid3X3 },
  { value: 8, abbr: 'CSP', name: 'Cyclic Sync Position', icon: CircleDot },
  { value: 9, abbr: 'CSV', name: 'Cyclic Sync Velocity', icon: Zap },
  { value: 10, abbr: 'CST', name: 'Cyclic Sync Torque', icon: Zap },
] as const;

export function ModeSelector({ selectedMode, onModeChange, disabled = false }: ModeSelectorProps) {
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-1.5">
        <Grid3X3 className="w-3.5 h-3.5 text-muted-foreground" />
        <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Operation Mode
        </span>
      </div>

      <div className="grid grid-cols-4 gap-1.5">
        {MODES.map((mode) => {
          const isSelected = mode.value === selectedMode;
          const Icon = mode.icon;

          return (
            <button
              key={mode.value}
              disabled={disabled}
              onClick={() => onModeChange(mode.value)}
              title={`${mode.value}: ${mode.name}`}
              className={cn(
                'flex flex-col items-center gap-0.5 px-1.5 py-2 rounded border text-xs transition-all',
                isSelected
                  ? 'bg-primary text-primary-foreground border-primary/50 shadow-sm'
                  : 'bg-card text-muted-foreground border-border hover:bg-muted hover:text-foreground',
                disabled && 'opacity-50 cursor-not-allowed',
              )}
            >
              <Icon className="w-3.5 h-3.5" />
              <span className="font-semibold leading-none">{mode.abbr}</span>
            </button>
          );
        })}
      </div>

      {/* Selected mode label */}
      <div className="text-[11px] text-muted-foreground">
        {selectedMode ? (
          <>
            Mode <span className="text-foreground font-medium">{selectedMode}</span>
            {' — '}
            {MODES.find((m) => m.value === selectedMode)?.name ?? 'Unknown'}
          </>
        ) : (
          'No mode selected'
        )}
      </div>
    </div>
  );
}
