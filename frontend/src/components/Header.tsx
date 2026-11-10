import React from 'react';
import { Download } from 'lucide-react';
import { GlassButton } from './ui/GlassButton';
import { GithubIcon } from './ui/GithubIcon';

export function Header() {
  const scrollToInstall = () => {
    document.getElementById('install-section')?.scrollIntoView({ behavior: 'smooth' });
  };

  return (
    <header className="sticky top-4 z-50 max-w-5xl mx-auto px-4 sm:px-6">
      <nav
        aria-label="Main Navigation"
        className="liquid-glass rounded-full px-4 sm:px-6 py-3 flex items-center justify-between gap-4"
      >
        {/* Brand */}
        <a
          href="#"
          className="flex items-center gap-3 group focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#6f9dff]/50 rounded-full"
        >
          <img
            src="/listune-logo.png"
            alt="Listune Studio logo"
            className="size-8 sm:size-9 rounded-full object-cover ring-1 ring-white/20 transition-transform duration-300 group-hover:scale-105"
          />
          <div className="flex flex-col">
            <span className="font-bold text-sm sm:text-base tracking-tight text-white">
              Listune Studio
            </span>
          </div>
        </a>

        {/* Desktop Links */}
        <div className="hidden md:flex items-center gap-6 text-sm text-white/70">
          <button
            type="button"
            onClick={scrollToInstall}
            className="hover:text-white transition-colors cursor-pointer"
          >
            Install
          </button>
          <a
            href="#features"
            className="hover:text-white transition-colors cursor-pointer"
          >
            Features
          </a>
          <a
            href="#faq"
            className="hover:text-white transition-colors cursor-pointer"
          >
            FAQ
          </a>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-2 sm:gap-3">
          <a
            href="https://github.com/lrmn7/listune-studio"
            target="_blank"
            rel="noreferrer"
            aria-label="GitHub Repository"
            className="p-2 sm:px-3 sm:py-2 rounded-full text-white/70 hover:text-white hover:bg-white/[0.08] transition-colors border border-transparent hover:border-white/10"
          >
            <GithubIcon className="size-4 sm:size-4.5" />
          </a>
          <GlassButton
            variant="accent"
            size="sm"
            onClick={scrollToInstall}
            className="hidden sm:inline-flex"
          >
            <Download className="size-3.5" />
            <span>Install App</span>
          </GlassButton>
        </div>
      </nav>
    </header>
  );
}
