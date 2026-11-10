import React, { useState } from 'react';
import {
  Apple,
  Monitor,
  Terminal,
  Download,
  FolderGit2,
  ExternalLink,
  ShieldCheck,
  CheckCircle2,
} from 'lucide-react';
import { GlassCard } from './ui/GlassCard';
import { GlassTabs, type TabItem } from './ui/GlassTabs';
import { CopyButton } from './ui/CopyButton';
import { GlassButton } from './ui/GlassButton';

export function InstallSection() {
  const [activePlatform, setActivePlatform] = useState<string>('macos');

  const tabs: TabItem[] = [
    { id: 'macos', label: 'macOS', icon: <Apple className="size-4" /> },
    { id: 'windows', label: 'Windows', icon: <Monitor className="size-4" /> },
    { id: 'linux', label: 'Linux', icon: <Terminal className="size-4" /> },
    { id: 'source', label: 'From Source', icon: <FolderGit2 className="size-4" /> },
  ];

  return (
    <section id="install-section" className="py-12 max-w-4xl mx-auto px-4 sm:px-6">
      <div className="text-center mb-8">
        <h2 className="text-2xl sm:text-4xl font-bold tracking-tight text-white mb-3">
          Installation Guide
        </h2>
        <p className="text-white/60 text-sm sm:text-base max-w-xl mx-auto">
          Choose your operating system to install the latest release with one command
          or download the standalone installer directly.
        </p>
      </div>

      {/* Liquid Glass Platform Tabs */}
      <div className="flex justify-center mb-8">
        <GlassTabs
          items={tabs}
          activeId={activePlatform}
          onChange={setActivePlatform}
          className="shadow-2xl"
        />
      </div>

      {/* Platform Content Panels */}
      <div className="space-y-6">
        {/* macOS */}
        {activePlatform === 'macos' && (
          <GlassCard className="p-6 sm:p-8 space-y-6">
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-mono text-white/50 uppercase tracking-wider">
                  Quick Install (Terminal)
                </span>
                <span className="text-xs text-white/40">Universal (Apple Silicon & Intel)</span>
              </div>
              <div className="liquid-glass-subtle p-3.5 sm:p-4 rounded-xl flex items-center justify-between gap-3 font-mono text-xs sm:text-sm text-[#93c5fd] overflow-x-auto">
                <span className="select-all break-all">
                  curl -fsSL https://studio.listune.app/install.sh | sh
                </span>
                <CopyButton
                  textToCopy="curl -fsSL https://studio.listune.app/install.sh | sh"
                  className="shrink-0"
                />
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-3 pt-2 border-t border-white/10">
              <a
                href="https://github.com/lrmn7/listune-studio/releases"
                target="_blank"
                rel="noreferrer"
              >
                <GlassButton variant="accent" size="md">
                  <Download className="size-4" />
                  <span>Download .dmg</span>
                </GlassButton>
              </a>
              <span className="text-xs text-white/50">
                Installs to <code className="text-white/80">/Applications</code> and keeps itself updated.
              </span>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2 text-xs text-white/65">
              <div className="liquid-glass-subtle p-3.5 rounded-xl">
                <div className="font-semibold text-white/90 mb-1">Custom Directory</div>
                Set <code className="text-[#93c5fd]">LISTUNE_INSTALL_DIR</code> before running curl to change destination folder.
              </div>
              <div className="liquid-glass-subtle p-3.5 rounded-xl">
                <div className="font-semibold text-white/90 mb-1">Pin Specific Version</div>
                Set <code className="text-[#93c5fd]">LISTUNE_VERSION=0.0.6</code> to install a specific historical release.
              </div>
            </div>
          </GlassCard>
        )}

        {/* Windows */}
        {activePlatform === 'windows' && (
          <GlassCard className="p-6 sm:p-8 space-y-6">
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-mono text-white/50 uppercase tracking-wider">
                  Quick Install (PowerShell)
                </span>
                <span className="text-xs text-white/40">Windows 10 / 11 (x64)</span>
              </div>
              <div className="liquid-glass-subtle p-3.5 sm:p-4 rounded-xl flex items-center justify-between gap-3 font-mono text-xs sm:text-sm text-[#93c5fd] overflow-x-auto">
                <span className="select-all break-all">
                  irm https://studio.listune.app/install.ps1 | iex
                </span>
                <CopyButton
                  textToCopy="irm https://studio.listune.app/install.ps1 | iex"
                  className="shrink-0"
                />
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-3 pt-2 border-t border-white/10">
              <a
                href="https://github.com/lrmn7/listune-studio/releases"
                target="_blank"
                rel="noreferrer"
              >
                <GlassButton variant="accent" size="md">
                  <Download className="size-4" />
                  <span>Download .exe Installer</span>
                </GlassButton>
              </a>
              <span className="text-xs text-white/50">
                User-level installation: no administrator prompt or UAC bypass required.
              </span>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2 text-xs text-white/65">
              <div className="liquid-glass-subtle p-3.5 rounded-xl">
                <div className="font-semibold text-white/90 mb-1">No Admin Rights Needed</div>
                Installs directly into your local user directory and configures your Start Menu shortcuts safely.
              </div>
              <div className="liquid-glass-subtle p-3.5 rounded-xl">
                <div className="font-semibold text-white/90 mb-1">Silent Automated Updates</div>
                Seamless background checks notify you when a new release is available with zero telemetry.
              </div>
            </div>
          </GlassCard>
        )}

        {/* Linux */}
        {activePlatform === 'linux' && (
          <GlassCard className="p-6 sm:p-8 space-y-6">
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-mono text-white/50 uppercase tracking-wider">
                  Quick Install (Bash / Sh)
                </span>
                <span className="text-xs text-white/40">x86_64 distributions</span>
              </div>
              <div className="liquid-glass-subtle p-3.5 sm:p-4 rounded-xl flex items-center justify-between gap-3 font-mono text-xs sm:text-sm text-[#93c5fd] overflow-x-auto">
                <span className="select-all break-all">
                  curl -fsSL https://studio.listune.app/install.sh | sh
                </span>
                <CopyButton
                  textToCopy="curl -fsSL https://studio.listune.app/install.sh | sh"
                  className="shrink-0"
                />
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-2.5 pt-2 border-t border-white/10">
              <a
                href="https://github.com/lrmn7/listune-studio/releases"
                target="_blank"
                rel="noreferrer"
              >
                <GlassButton variant="accent" size="sm">
                  <Download className="size-3.5" />
                  <span>.deb (Debian / Ubuntu)</span>
                </GlassButton>
              </a>
              <a
                href="https://github.com/lrmn7/listune-studio/releases"
                target="_blank"
                rel="noreferrer"
              >
                <GlassButton variant="glass" size="sm">
                  <Download className="size-3.5" />
                  <span>.rpm (Fedora / RHEL)</span>
                </GlassButton>
              </a>
              <a
                href="https://github.com/lrmn7/listune-studio/releases"
                target="_blank"
                rel="noreferrer"
              >
                <GlassButton variant="glass" size="sm">
                  <Download className="size-3.5" />
                  <span>.AppImage (Universal)</span>
                </GlassButton>
              </a>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2 text-xs text-white/65">
              <div className="liquid-glass-subtle p-3.5 rounded-xl">
                <div className="font-semibold text-white/90 mb-1">Package Managers Supported</div>
                Auto-detects <code className="text-[#93c5fd]">apt</code>, <code className="text-[#93c5fd]">dnf</code>, or <code className="text-[#93c5fd]">zypper</code>. Uses <code className="text-white/80">~/.local/share</code> if sudo is absent.
              </div>
              <div className="liquid-glass-subtle p-3.5 rounded-xl">
                <div className="font-semibold text-white/90 mb-1">WebKitGTK Requirements</div>
                Optimized for WebKitGTK 2.40+. The AppImage format requires FUSE 2.
              </div>
            </div>
          </GlassCard>
        )}

        {/* Source */}
        {activePlatform === 'source' && (
          <GlassCard className="p-6 sm:p-8 space-y-6">
            <div>
              <div className="text-xs font-mono text-white/50 uppercase tracking-wider mb-2">
                Prerequisites: Node.js 22+, pnpm, Rust toolchain, FFmpeg
              </div>
              <div className="space-y-3">
                <div className="liquid-glass-subtle p-3.5 sm:p-4 rounded-xl flex items-center justify-between gap-3 font-mono text-xs sm:text-sm text-[#93c5fd] overflow-x-auto">
                  <span className="select-all">git clone https://github.com/lrmn7/listune-studio.git</span>
                  <CopyButton textToCopy="git clone https://github.com/lrmn7/listune-studio.git" />
                </div>
                <div className="liquid-glass-subtle p-3.5 sm:p-4 rounded-xl flex items-center justify-between gap-3 font-mono text-xs sm:text-sm text-[#93c5fd] overflow-x-auto">
                  <span className="select-all">cd listune-studio && pnpm install</span>
                  <CopyButton textToCopy="cd listune-studio && pnpm install" />
                </div>
                <div className="liquid-glass-subtle p-3.5 sm:p-4 rounded-xl flex items-center justify-between gap-3 font-mono text-xs sm:text-sm text-[#93c5fd] overflow-x-auto">
                  <span className="select-all">pnpm dev:desktop</span>
                  <CopyButton textToCopy="pnpm dev:desktop" />
                </div>
              </div>
            </div>

            <div className="pt-2 border-t border-white/10 flex flex-wrap items-center justify-between gap-4">
              <span className="text-xs text-white/60">
                Launches the Tauri desktop app with hot reload in development mode.
              </span>
              <a
                href="https://github.com/lrmn7/listune-studio/blob/main/docs/development.md"
                target="_blank"
                rel="noreferrer"
                className="text-xs text-[#6f9dff] hover:underline inline-flex items-center gap-1"
              >
                <span>Read development documentation</span>
                <ExternalLink className="size-3" />
              </a>
            </div>
          </GlassCard>
        )}
      </div>
    </section>
  );
}

