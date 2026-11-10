import React, { useState } from 'react';
import { Copy, Check } from 'lucide-react';
import { clsx } from 'clsx';

interface CopyButtonProps {
  textToCopy: string;
  label?: string;
  className?: string;
}

export function CopyButton({ textToCopy, label, className }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(textToCopy);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy to clipboard', err);
    }
  };

  return (
    <button
      type="button"
      onClick={handleCopy}
      aria-label={copied ? 'Copied to clipboard' : 'Copy command to clipboard'}
      title={copied ? 'Copied' : 'Copy'}
      className={clsx(
        'relative inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-medium cursor-pointer',
        'border transition-all duration-200 select-none min-h-[36px] min-w-[36px]',
        copied
          ? 'bg-emerald-500/20 text-emerald-300 border-emerald-400/30'
          : 'bg-white/[0.08] hover:bg-white/[0.14] text-white/80 hover:text-white border-white/15',
        className
      )}
    >
      {copied ? (
        <>
          <Check className="size-3.5 text-emerald-400" />
          {label && <span>Copied</span>}
        </>
      ) : (
        <>
          <Copy className="size-3.5" />
          {label && <span>{label}</span>}
        </>
      )}
    </button>
  );
}

