import React from 'react';
import { Cpu, Film, Shield, Terminal } from 'lucide-react';
import { GlassCard } from './ui/GlassCard';

export function FeaturesGrid() {
  const features = [
    {
      icon: <Cpu className="size-5 text-[#6f9dff]" />,
      title: 'Bring Your Own Agent',
      desc: 'Connects directly over standard Agent Control Protocol (ACP). Use Claude Code, opencode, Antigravity CLI, Codex, or your custom coding agent.',
    },
    {
      icon: <Film className="size-5 text-[#c084fc]" />,
      title: '28 Motion Graphics Styles',
      desc: 'Includes curated video styles from Atlas Journey to Swiss Motion. All preview videos and poster frames run 100% locally from bundled assets.',
    },
    {
      icon: <Shield className="size-5 text-[#34d399]" />,
      title: 'Zero Telemetry & Private',
      desc: 'All project data, styles, and briefs remain on your disk in a local SQLite database. We never collect analytics or phone home.',
    },
    {
      icon: <Terminal className="size-5 text-[#38bdf8]" />,
      title: 'Explicit Permissions',
      desc: 'Every file edit and bash command proposed by your agent asks for your explicit permission first, unless you explicitly enable Auto mode.',
    },
  ];

  return (
    <section id="features" className="py-12 max-w-4xl mx-auto px-4 sm:px-6">
      <div className="text-center mb-8">
        <h2 className="text-2xl sm:text-3xl font-bold tracking-tight text-white mb-2">
          Engineered for Local Creative Coding
        </h2>
        <p className="text-white/60 text-sm max-w-lg mx-auto">
          Combining the power of Remotion with autonomous AI coding agents in a native desktop environment.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {features.map((f, i) => (
          <GlassCard key={i} variant="interactive" className="p-6">
            <div className="p-2.5 rounded-xl bg-white/[0.06] border border-white/10 inline-flex mb-4">
              {f.icon}
            </div>
            <h3 className="text-base font-semibold text-white mb-1.5">{f.title}</h3>
            <p className="text-sm text-white/65 leading-relaxed">{f.desc}</p>
          </GlassCard>
        ))}
      </div>
    </section>
  );
}

