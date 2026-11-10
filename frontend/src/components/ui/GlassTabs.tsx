import React from 'react';
import { clsx } from 'clsx';

export interface TabItem {
  id: string;
  label: string;
  icon?: React.ReactNode;
}

interface GlassTabsProps {
  items: TabItem[];
  activeId: string;
  onChange: (id: string) => void;
  className?: string;
}

export function GlassTabs({
  items,
  activeId,
  onChange,
  className,
}: GlassTabsProps) {
  return (
    <div
      role="tablist"
      aria-label="Platform selection"
      className={clsx(
        'relative inline-flex items-center p-1.5 rounded-full',
        'bg-black/35 backdrop-blur-xl border border-white/10 shadow-[inset_0_1px_2px_rgba(0,0,0,0.6)]',
        className
      )}
    >
      {items.map((tab) => {
        const isActive = tab.id === activeId;
        return (
          <button
            key={tab.id}
            role="tab"
            aria-selected={isActive}
            tabIndex={isActive ? 0 : -1}
            onClick={() => onChange(tab.id)}
            className={clsx(
              'relative z-10 flex items-center justify-center gap-2 px-5 py-2.5 rounded-full text-xs sm:text-sm font-medium transition-all duration-200 select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#6f9dff]/50 min-h-[40px]',
              isActive
                ? 'text-white'
                : 'text-white/60 hover:text-white hover:bg-white/[0.04]'
            )}
          >
            {tab.icon && <span className="shrink-0">{tab.icon}</span>}
            <span>{tab.label}</span>

            {/* Active Liquid Glass Lens Pill */}
            {isActive && (
              <span
                className="absolute inset-0 -z-10 rounded-full border border-white/25 shadow-[0_4px_16px_rgba(0,0,0,0.35),inset_0_1px_1px_rgba(255,255,255,0.4)]"
                style={{
                  background:
                    'linear-gradient(180deg, rgba(255, 255, 255, 0.18) 0%, rgba(255, 255, 255, 0.05) 100%)',
                }}
              />
            )}
          </button>
        );
      })}
    </div>
  );
}

