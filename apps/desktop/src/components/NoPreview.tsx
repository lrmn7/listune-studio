import type { ReactNode } from 'react';
import { Film } from 'lucide-react';
import { cn } from '@/lib/utils';

const SWATCHES: [string, string][] = [
  ['#0B1E3F', '#FFFFFF'],
  ['#F4688F', '#141414'],
  ['#E8ECF4', '#0B1E3F'],
  ['#3B1F14', '#FFE7D6'],
  ['#1F2A1A', '#F5E6A8'],
  ['#FFF0E0', '#3B1F14'],
];

/** A colour pair picked from `id`, the same every time. */
export function swatch(id: string) {
  let h = 0;
  for (const c of id) h = (h * 31 + c.charCodeAt(0)) >>> 0;
  return SWATCHES[h % SWATCHES.length]!;
}

function hexLuminance(hex: string): number | null {
  const clean = hex.trim().replace(/^#/, '');
  const full =
    clean.length === 3
      ? clean
          .split('')
          .map((c) => c + c)
          .join('')
      : clean.length === 6
        ? clean
        : null;
  if (!full || !/^[0-9a-fA-F]{6}$/.test(full)) return null;
  const toLinear = (v: number) => {
    const s = v / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  const r = toLinear(parseInt(full.slice(0, 2), 16));
  const g = toLinear(parseInt(full.slice(2, 4), 16));
  const b = toLinear(parseInt(full.slice(4, 6), 16));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrastInk(colors: string[]): string | undefined {
  const lums = colors
    .slice(0, 2)
    .map(hexLuminance)
    .filter((n): n is number => n !== null);
  if (lums.length === 0) return undefined;
  const avg = lums.reduce((a, b) => a + b, 0) / lums.length;
  return avg > 0.184 ? '#141414' : '#F5F5F5';
}

/**
 * The calm panel shown when there is no picture: a film icon on the `swatchId`'s colour (a style), or on ink
 * (a video). `children` is the muted line under the icon; without it the panel is decoration only.
 */
export function NoPreview({
  swatchId,
  palette,
  className,
  'data-testid': testId = 'no-preview',
  children,
}: {
  swatchId?: string;
  palette?: string[];
  className?: string;
  'data-testid'?: string;
  children?: ReactNode;
}) {
  const hasPalette = Boolean(palette && palette.length > 0);
  const bg = hasPalette
    ? palette!.length >= 2
      ? `linear-gradient(135deg, ${palette![0]} 0%, ${palette![1]} 100%)`
      : palette![0]
    : swatchId
      ? swatch(swatchId)[0]
      : undefined;
  const fg = hasPalette ? contrastInk(palette!) : swatchId ? swatch(swatchId)[1] : undefined;

  return (
    <div
      data-testid={testId}
      className={cn(
        'flex aspect-[9/16] w-full flex-col items-center justify-center gap-2 rounded-md border border-[#333]',
        !swatchId && !hasPalette && 'bg-ink',
        className,
      )}
      style={bg ? { background: bg } : undefined}
      aria-hidden={children ? undefined : 'true'}
    >
      <Film
        size={24}
        strokeWidth={1.5}
        aria-hidden="true"
        className={fg ? undefined : 'text-text-3'}
        style={fg ? { color: fg } : undefined}
      />
      {children && (
        <span
          className={cn('inline-flex items-center gap-1.5 text-xs', !fg && 'text-text-3')}
          style={fg ? { color: fg } : undefined}
        >
          {children}
        </span>
      )}
    </div>
  );
}
