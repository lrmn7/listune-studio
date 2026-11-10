import logo from '@/assets/brand/listune-logo.png';
import { cn } from '@/lib/utils';

export type BrandMood = 'default' | 'wink' | 'happy' | 'sleepy' | 'surprised';

const MOOD_CLASS: Record<BrandMood, string> = {
  default: '',
  wink: '',
  happy: 'brightness-105',
  sleepy: 'opacity-80',
  surprised: 'rounded-full ring-2 ring-primary/50',
};

export function BrandMark({
  mood = 'default',
  size = 28,
  className,
}: {
  mood?: BrandMood;
  size?: number;
  className?: string;
}) {
  return (
    <img
      src={logo}
      alt="Listune"
      width={size}
      height={size}
      style={{ width: size, height: size }}
      className={cn('inline-block shrink-0 select-none', MOOD_CLASS[mood], className)}
    />
  );
}
