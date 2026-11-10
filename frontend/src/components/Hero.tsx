import React from 'react';
import { Download, Terminal } from 'lucide-react';
import { GlassButton } from './ui/GlassButton';

export function Hero() {
  const scrollToInstall = () => {
    document.getElementById('install-section')?.scrollIntoView({ behavior: 'smooth' });
  };

  const scrollToSource = () => {
    document.getElementById('install-section')?.scrollIntoView({ behavior: 'smooth' });
    const sourceTabBtn = document.querySelector('[data-platform-tab="source"]') as HTMLButtonElement;
    if (sourceTabBtn) sourceTabBtn.click();
  };

  return (
    <section className="pt-16 sm:pt-24 pb-12 sm:pb-16 text-center max-w-4xl mx-auto px-4 sm:px-6">
      {/* Title - strictly NO badge above it */}
      <h1 className="text-4xl sm:text-6xl md:text-7xl font-extrabold tracking-tight text-white mb-6 leading-[1.1]">
        Listune Studio
      </h1>

      {/* Subtitle */}
      <p className="text-base sm:text-xl text-white/75 max-w-2xl mx-auto mb-10 leading-relaxed font-normal">
        A lightweight desktop studio for creating motion videos and animated graphics
        with Remotion and your AI coding agent. 100% local, offline-ready, and private.
      </p>

      {/* Hero CTA buttons */}
      <div className="flex flex-wrap items-center justify-center gap-3 sm:gap-4 mb-12">
        <GlassButton variant="accent" size="lg" onClick={scrollToInstall}>
          <Download className="size-4.5" />
          <span>Quick Install</span>
        </GlassButton>
        <GlassButton variant="glass" size="lg" onClick={scrollToSource}>
          <Terminal className="size-4.5" />
          <span>Run from Source</span>
        </GlassButton>
      </div>

      {/* Key value pillars */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 max-w-3xl mx-auto text-left">
        <div className="liquid-glass-subtle p-3.5 rounded-2xl">
          <div className="text-xs text-white/50 mb-1">Architecture</div>
          <div className="text-sm font-semibold text-white">Local-First Studio</div>
        </div>
        <div className="liquid-glass-subtle p-3.5 rounded-2xl">
          <div className="text-xs text-white/50 mb-1">Motion Engine</div>
          <div className="text-sm font-semibold text-white">Remotion Built-in</div>
        </div>
        <div className="liquid-glass-subtle p-3.5 rounded-2xl">
          <div className="text-xs text-white/50 mb-1">AI Protocol</div>
          <div className="text-sm font-semibold text-white">Bring Your Agent</div>
        </div>
        <div className="liquid-glass-subtle p-3.5 rounded-2xl">
          <div className="text-xs text-white/50 mb-1">Privacy</div>
          <div className="text-sm font-semibold text-white">Zero Telemetry</div>
        </div>
      </div>
    </section>
  );
}

