/**
 * StateMachine — DS402 state machine visualization component.
 *
 * Renders the 7 standard CiA 402 device profile states as labeled boxes
 * with the current state highlighted. Below the state boxes, available
 * transitions from the current state are listed as actionable items.
 *
 * The component is compact enough to fit in a detail panel (~300px height).
 */

import { useMemo } from 'react';
import { ArrowRight, AlertTriangle, Zap, Power, Settings } from 'lucide-react';
import { cn } from '@/lib/utils';

export interface StateMachineProps {
  /** Current DS402 state name */
  currentState: string;
  /** Optional callback when a transition is triggered */
  onTransition?: (transition: string) => void;
  /** Whether the component is in read-only mode */
  readOnly?: boolean;
}

/** All 7 standard DS402 states */
const DS402_STATES = [
  {
    key: 'Not Ready to Switch On',
    short: 'Not Ready',
    icon: AlertTriangle,
  },
  {
    key: 'Switch On Disabled',
    short: 'SW Disabled',
    icon: Power,
  },
  {
    key: 'Ready to Switch On',
    short: 'Ready',
    icon: Settings,
  },
  {
    key: 'Switched On',
    short: 'Switched On',
    icon: Zap,
  },
  {
    key: 'Operation Enabled',
    short: 'Op Enabled',
    icon: Zap,
  },
  {
    key: 'Fault Reaction Active',
    short: 'Fault React',
    icon: AlertTriangle,
  },
  {
    key: 'Fault',
    short: 'Fault',
    icon: AlertTriangle,
  },
] as const;

/** Standard DS402 transitions keyed by source state */
const TRANSITIONS: Record<string, { label: string; target: string; icon: typeof ArrowRight }[]> = {
  'Not Ready to Switch On': [
    { label: 'auto → Switch On Disabled', target: 'Switch On Disabled', icon: ArrowRight },
  ],
  'Switch On Disabled': [
    { label: 'auto → Not Ready', target: 'Not Ready to Switch On', icon: ArrowRight },
    { label: 'shutdown → Ready', target: 'Ready to Switch On', icon: ArrowRight },
  ],
  'Ready to Switch On': [
    { label: 'switch on → Switched On', target: 'Switched On', icon: ArrowRight },
    { label: 'disable voltage → SW Disabled', target: 'Switch On Disabled', icon: ArrowRight },
  ],
  'Switched On': [
    { label: 'enable op → Op Enabled', target: 'Operation Enabled', icon: ArrowRight },
    { label: 'shutdown → Ready', target: 'Ready to Switch On', icon: ArrowRight },
    { label: 'disable voltage → SW Disabled', target: 'Switch On Disabled', icon: ArrowRight },
  ],
  'Operation Enabled': [
    { label: 'disable op → Switched On', target: 'Switched On', icon: ArrowRight },
    { label: 'shutdown → Ready', target: 'Ready to Switch On', icon: ArrowRight },
    { label: 'quick stop → SW Disabled', target: 'Switch On Disabled', icon: ArrowRight },
  ],
  'Fault Reaction Active': [
    { label: 'auto → Not Ready', target: 'Not Ready to Switch On', icon: ArrowRight },
  ],
  'Fault': [
    { label: 'fault reset → SW Disabled', target: 'Switch On Disabled', icon: ArrowRight },
  ],
};

/** Get state-specific styling based on the state key */
function getStateStyle(stateKey: string, isActive: boolean): { bg: string; border: string; text: string; badge: string } {
  if (isActive) {
    switch (stateKey) {
      case 'Operation Enabled':
        return {
          bg: 'bg-green-500/15',
          border: 'border-green-500/50',
          text: 'text-green-400',
          badge: 'bg-green-500/20 text-green-400',
        };
      case 'Ready to Switch On':
      case 'Switched On':
        return {
          bg: 'bg-yellow-500/15',
          border: 'border-yellow-500/50',
          text: 'text-yellow-400',
          badge: 'bg-yellow-500/20 text-yellow-400',
        };
      case 'Fault':
      case 'Fault Reaction Active':
        return {
          bg: 'bg-red-500/15',
          border: 'border-red-500/50',
          text: 'text-red-400',
          badge: 'bg-red-500/20 text-red-400',
        };
      case 'Not Ready to Switch On':
        return {
          bg: 'bg-orange-500/15',
          border: 'border-orange-500/50',
          text: 'text-orange-400',
          badge: 'bg-orange-500/20 text-orange-400',
        };
      default:
        return {
          bg: 'bg-muted',
          border: 'border-border',
          text: 'text-muted-foreground',
          badge: 'bg-muted-foreground/20 text-muted-foreground',
        };
    }
  }

  return {
    bg: 'bg-card',
    border: 'border-border',
    text: 'text-muted-foreground',
    badge: 'bg-muted text-muted-foreground',
  };
}

export function StateMachine({
  currentState,
  onTransition,
  readOnly = false,
}: StateMachineProps) {
  const availableTransitions = useMemo(
    () => TRANSITIONS[currentState] ?? [],
    [currentState],
  );

  return (
    <div className="space-y-2">
      {/* State machine header */}
      <div className="flex items-center gap-1.5">
        <Settings className="w-3.5 h-3.5 text-muted-foreground" />
        <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          State Machine
        </span>
      </div>

      {/* States list */}
      <div className="space-y-0.5">
        {DS402_STATES.map((state) => {
          const isActive = state.key === currentState;
          const style = getStateStyle(state.key, isActive);
          const Icon = state.icon;

          return (
            <div
              key={state.key}
              className={cn(
                'flex items-center gap-2 px-2 py-1.5 rounded border text-xs transition-all',
                style.bg,
                style.border,
                isActive && 'ring-1 ring-offset-0',
              )}
            >
              <Icon className={cn('w-3 h-3 shrink-0', isActive ? style.text : 'text-muted-foreground')} />
              <span className={cn('font-medium', isActive ? 'text-foreground' : 'text-muted-foreground')}>
                {state.key}
              </span>
              {isActive && (
                <span className={cn('ml-auto px-1.5 py-0.5 rounded text-[10px] font-medium', style.badge)}>
                  ACTIVE
                </span>
              )}
            </div>
          );
        })}
      </div>

      {/* Available transitions */}
      {availableTransitions.length > 0 && (
        <div className="pt-2 border-t border-border">
          <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
            Available Transitions
          </span>
          <div className="mt-1 space-y-0.5">
            {availableTransitions.map((t, i) => {
              const Icon = t.icon;
              return (
                <button
                  key={i}
                  disabled={readOnly || !onTransition}
                  onClick={() => onTransition?.(t.label)}
                  className={cn(
                    'flex items-center gap-1.5 w-full px-2 py-1 rounded text-[11px] transition-colors',
                    readOnly || !onTransition
                      ? 'text-muted-foreground cursor-default'
                      : 'text-muted-foreground hover:bg-muted hover:text-foreground cursor-pointer',
                  )}
                >
                  <Icon className="w-3 h-3" />
                  <span>{t.label}</span>
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
