import { useEffect } from 'react';
import { useAppStore } from '@/lib/store';

/**
 * Global keyboard shortcuts for OpenCAN.
 *
 * Shortcuts:
 * - Ctrl/Cmd+K: Toggle connection dialog
 * - Ctrl/Cmd+L: Clear frames
 * - Space: Pause/resume frame stream
 * - Ctrl/Cmd+1-4: Switch navigation groups (CAN/CANOpen/Recording/EDS)
 * - Escape: Close dialogs/panels
 */
export function useKeyboardShortcuts() {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isModifier = e.ctrlKey || e.metaKey;
      const target = e.target as HTMLElement;
      // Don't trigger shortcuts when typing in inputs
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT') {
        return;
      }

      // Ctrl/Cmd+K: Toggle connection dialog
      if (isModifier && e.key === 'k') {
        e.preventDefault();
        const state = useAppStore.getState();
        if (state.connectionDialog.visible) {
          state.connectionDialog.hide();
        } else {
          state.connectionDialog.show();
        }
        return;
      }

      // Ctrl/Cmd+L: Clear frames
      if (isModifier && e.key === 'l') {
        e.preventDefault();
        useAppStore.getState().frames.clearFrames();
        return;
      }

      // Space: Pause/resume (only when not in an input)
      if (e.key === ' ' && !isModifier) {
        e.preventDefault();
        useAppStore.setState((s) => ({
          ui: { ...s.ui, paused: !s.ui.paused },
        }));
        return;
      }

      // Ctrl/Cmd+1-4: Switch navigation groups
      if (isModifier && ['1', '2', '3', '4'].includes(e.key)) {
        e.preventDefault();
        const groups = ['can', 'canopen', 'recording', 'eds'] as const;
        const idx = parseInt(e.key) - 1;
        const group = groups[idx];
        if (group) {
          const { setActiveGroup } = useAppStore.getState().sidebar;
          setActiveGroup(group);
        }
        return;
      }

      // Escape: Close connection dialog
      if (e.key === 'Escape') {
        const state = useAppStore.getState();
        if (state.connectionDialog.visible) {
          state.connectionDialog.hide();
        }
        return;
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, []);
}
