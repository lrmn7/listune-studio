import { create } from 'zustand';
import type { Settings } from '@listune/shared';

export type Route =
  | { name: 'welcome' }
  | { name: 'connect' }
  | { name: 'requirements' }
  | { name: 'chat-new'; styleId?: string }
  | {
      name: 'chat';
      chatId: string;
      /** Prefilled in the composer, never sent. */
      draft?: string;
      /** A chat without a project that New chat started from a template on the Style screen. */
      styleId?: string;
    }
  | { name: 'videos' }
  | { name: 'style'; /** Focus the link box when the screen opens. */ focusLink?: boolean }
  | { name: 'agents' }
  | { name: 'settings'; /** The tab to open on (Settings' own id). */ tab?: string };

/** Asked before every navigation while set; returning false keeps the current screen (it asks the user itself). */
export type LeaveGuard = (next: Route) => boolean;

interface AppState {
  route: Route;
  navigate(r: Route): void;
  leaveGuard: LeaveGuard | null;
  setLeaveGuard(guard: LeaveGuard | null): void;
  settings: Settings | null;
  setSettings(s: Settings): void;
  needsYouCount: number;
  /** The chat New chat started last in this app session: Chat goes back to it while it runs. */
  newChatId: string | null;
  /** The chat New chat just handed its first message to: its composer takes the focus over. */
  focusChatId: string | null;
  /** Bumped when a project is created so the sidebar's Recent list refreshes. */
  projectsVersion: number;
  bumpProjects(): void;
}

export const useApp = create<AppState>((set, get) => ({
  route: { name: 'welcome' },
  navigate: (route) => {
    const guard = get().leaveGuard;
    if (guard && !guard(route)) return;
    set({ route });
  },
  leaveGuard: null,
  setLeaveGuard: (leaveGuard) => set({ leaveGuard }),
  settings: null,
  setSettings: (settings) => set({ settings }),
  needsYouCount: 0,
  newChatId: null,
  focusChatId: null,
  projectsVersion: 0,
  bumpProjects: () => set((s) => ({ projectsVersion: s.projectsVersion + 1 })),
}));
