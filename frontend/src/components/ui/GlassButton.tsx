import React from 'react';
import { clsx } from 'clsx';

interface GlassButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'glass' | 'accent' | 'subtle';
  size?: 'sm' | 'md' | 'lg';
  children: React.ReactNode;
}

export function GlassButton({
  variant = 'glass',
  size = 'md',
  className,
  children,
  ...props
}: GlassButtonProps) {
  return (
    <button
      className={clsx(
        'relative inline-flex items-center justify-center font-medium rounded-full cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#6f9dff]/60 disabled:opacity-50 disabled:pointer-events-none transition-all duration-200 select-none',
        // Sizes
        size === 'sm' && 'text-xs px-3.5 py-1.5 gap-1.5 min-h-[32px]',
        size === 'md' && 'text-sm px-5 py-2.5 gap-2 min-h-[44px]',
        size === 'lg' && 'text-base px-6 py-3.5 gap-2.5 min-h-[50px]',
        // Variants
        variant === 'glass' && 'liquid-glass-button text-[#f5f6ff]',
        variant === 'accent' && 'liquid-glass-accent text-white font-semibold',
        variant === 'subtle' &&
          'bg-white/[0.04] hover:bg-white/[0.08] text-[#f5f6ff]/80 hover:text-white border border-white/10 active:scale-[0.98]',
        className
      )}
      {...props}
    >
      {/* Specular highlight for accent button */}
      {variant === 'accent' && (
        <span className="absolute inset-x-3 top-0 h-[1px] bg-white/60 pointer-events-none rounded-full" />
      )}
      {children}
    </button>
  );
}

