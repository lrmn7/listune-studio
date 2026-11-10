import React from 'react';
import { clsx } from 'clsx';

interface GlassCardProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'subtle' | 'interactive';
  children: React.ReactNode;
}

export function GlassCard({
  variant = 'default',
  className,
  children,
  ...props
}: GlassCardProps) {
  return (
    <div
      className={clsx(
        'rounded-[24px] relative overflow-hidden',
        variant === 'default' && 'liquid-glass',
        variant === 'subtle' && 'liquid-glass-subtle',
        variant === 'interactive' && 'liquid-glass liquid-glass-interactive',
        className
      )}
      {...props}
    >
      {/* Specular top rim light for optical realism */}
      <div className="absolute inset-x-0 top-0 h-[1px] bg-gradient-to-r from-transparent via-white/40 to-transparent pointer-events-none" />
      {children}
    </div>
  );
}

