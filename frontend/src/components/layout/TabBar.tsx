// Secondary tab bar — shown below TopBar, switches with sidebar group selection

import { useAppStore, useGroupTabs } from '@/lib/store';

export function TabBar() {
  const tabs = useGroupTabs();
  const currentTab = useAppStore((s) => s.ui.currentTab);
  const setCurrentTab = useAppStore((s) => s.ui.setCurrentTab);

  return (
    <div className="flex items-center gap-0.5 px-3 py-1.5 border-b bg-card/50 shrink-0">
      {tabs.map((tab) => (
        <button
          key={tab.key}
          className={`px-2.5 py-1 text-xs rounded transition-colors ${
            currentTab === tab.key
              ? 'bg-primary text-primary-foreground font-medium'
              : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'
          }`}
          onClick={() => setCurrentTab(tab.key)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
