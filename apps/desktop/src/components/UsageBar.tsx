import { i18n } from '@/i18n';
import { AGENT_NAMES } from '@/lib/agents';
import { formatResetIn } from '@/lib/time';
import { cn } from '@/lib/utils';
import type { AgentUsage, Spend } from '@listune/shared';

export const pct = (n: number) => `${Math.round(n)}%`;

/** $0.00, or more decimals for a small but not zero cost ($0.0042). */
export function formatCost(usd: number): string {
  const digits = usd > 0 && usd < 0.01 ? 4 : 2;
  return new Intl.NumberFormat(i18n.language || 'en', { style: 'currency', currency: 'USD', minimumFractionDigits: digits, maximumFractionDigits: digits }).format(usd);
}

/** 186K, 1.2M: in the interface language. */
export const formatTokens = (n: number) => new Intl.NumberFormat(i18n.language || 'en', { notation: 'compact', maximumFractionDigits: 1 }).format(n);

/** `$0.00 · 262.3K tokens`; the reply count goes in `spendTitle`, so the row fits the popover. */
export const spendRow = (s: Spend) => i18n.t('usage.spendRow', { cost: formatCost(s.costUsd), tokens: formatTokens(s.tokens) });
export const spendTitle = (s: Spend) => i18n.t('usage.spendTitle', { count: s.replies });

/** Why there is no usage, in the interface language (the agent's own words only for errors). */
export function usageMessage(u: AgentUsage): string {
  if (u.status === 'unavailable') return i18n.t('usage.notYet', { agent: AGENT_NAMES[u.agentId] });
  return u.message ?? i18n.t('usage.unavailable');
}

/** `33% used 1h 41m · 48% used 3d 13h · 26% used Fable` */
export function usageLine(u: AgentUsage, now: Date = new Date()): string {
  const t = i18n.t;
  if (u.status !== 'ok') return usageMessage(u);
  if (u.spend) return t('usage.spendLine', { cost: formatCost(u.spend.today.costUsd), tokens: formatTokens(u.spend.today.tokens) });
  const parts: string[] = [];
  if (u.session) parts.push(t('usage.used', { pct: pct(u.session.usedPercent), what: formatResetIn(u.session.resetsAt, now) }).trim());
  if (u.weekly) parts.push(t('usage.used', { pct: pct(u.weekly.usedPercent), what: formatResetIn(u.weekly.resetsAt, now) }).trim());
  if (u.modelWeekly) parts.push(t('usage.used', { pct: pct(u.modelWeekly.window.usedPercent), what: u.modelWeekly.label }));
  return parts.join(' · ') || t('usage.none');
}

export function UsageMeter({ percent, label, className }: { percent: number; label: string; className?: string }) {
  const p = Math.max(0, Math.min(100, percent));
  return (
    <div
      role="progressbar"
      aria-label={label}
      aria-valuenow={Math.round(p)}
      aria-valuemin={0}
      aria-valuemax={100}
      className={cn('h-1 w-full overflow-hidden rounded-full bg-raised', className)}
    >
      <div className={cn('h-full rounded-full', p >= 80 ? 'bg-primary' : 'bg-text-2')} style={{ width: `${p}%` }} />
    </div>
  );
}
