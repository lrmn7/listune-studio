import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';
import { i18n } from '@/i18n';

/** Rust emits this to the main window every 30 s, also while the window is hidden. */
export const TICK_EVENT = 'scheduler://tick';

export type TickHandler = (now: Date) => void | Promise<void>;
const handlers = new Set<TickHandler>();
/** Handlers whose previous call has not finished yet. */
const inFlight = new WeakSet<TickHandler>();

/**
 * Runs `handler` on every tick (and once at startup). Returns the unsubscribe.
 * Handlers may be long-running: a handler is never re-entered while its previous call is pending (it just
 * skips those ticks), and a slow handler never delays other handlers. Errors are logged.
 */
export function onTick(handler: TickHandler): () => void {
  handlers.add(handler);
  return () => void handlers.delete(handler);
}

/** Whether notifications may show, asking the first time. */
export async function canNotify(): Promise<boolean> {
  if (await isPermissionGranted()) return true;
  return (await requestPermission()) === 'granted';
}

/**
 * Shows a system notification. Called through the plugin's command (not `sendNotification`, which fires
 * and forgets) so a failure can be caught. Note: on desktop the plugin always reports permission as
 * granted, so a user who turned Listune's notifications off in the OS is not detected; Settings has a
 * test button for checking them.
 */
export async function show(body: string): Promise<void> {
  await invoke('plugin:notification|notify', { options: { title: i18n.t('settings.notificationTitle'), body } });
}

/** Settings' test button: sends a notification now. Returns false when the system reports a denial. */
export async function sendTestNotification(): Promise<boolean> {
  const granted = await canNotify();
  if (!granted) return false;
  await show(i18n.t('settings.notificationTest'));
  return true;
}

/** One pass: runs every registered tick handler that is not still busy, each on its own. */
export async function tick(now: Date = new Date()): Promise<void> {
  for (const h of [...handlers]) {
    if (inFlight.has(h)) continue;
    inFlight.add(h);
    void Promise.resolve()
      .then(() => h(now))
      .catch((e) => console.error('Listune: tick handler failed', e))
      .finally(() => inFlight.delete(h));
  }
}

/** Subscribes to the Rust tick and runs a pass right away. */
export async function startScheduler(): Promise<UnlistenFn> {
  const unlisten = await listen<string>(TICK_EVENT, () => void tick());
  void tick();
  return unlisten;
}
