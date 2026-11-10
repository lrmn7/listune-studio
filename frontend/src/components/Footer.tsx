import React from 'react';
import { ExternalLink } from 'lucide-react';
import { GithubIcon } from './ui/GithubIcon';

export function Footer() {
  return (
    <footer className="mt-16 border-t border-white/10 relative z-10">
      <div className="max-w-5xl mx-auto px-4 sm:px-6 py-8 flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-white/50">
        <div className="flex items-center gap-2.5">
          <img
            src="/listune-logo.png"
            alt="Listune logo"
            className="size-5 rounded-full object-cover"
          />
          <span className="font-medium text-white/80">Listune Studio</span>
          <span>— Open-source local motion studio</span>
        </div>

        <div className="flex items-center gap-5 text-white/60">
          <a
            href="https://github.com/lrmn7/listune-studio"
            target="_blank"
            rel="noreferrer"
            className="hover:text-white transition-colors flex items-center gap-1"
          >
            <GithubIcon className="size-3.5" />
            <span>GitHub</span>
          </a>
          <a
            href="https://github.com/lrmn7/listune-studio/releases"
            target="_blank"
            rel="noreferrer"
            className="hover:text-white transition-colors flex items-center gap-1"
          >
            <span>Releases</span>
            <ExternalLink className="size-3" />
          </a>
          <a
            href="https://github.com/lrmn7/listune-studio/blob/main/docs/development.md"
            target="_blank"
            rel="noreferrer"
            className="hover:text-white transition-colors"
          >
            Docs
          </a>
        </div>
      </div>
    </footer>
  );
}
