import React, { useState } from 'react';
import { ChevronDown } from 'lucide-react';
import { GlassCard } from './ui/GlassCard';
import { clsx } from 'clsx';

interface FAQItem {
  question: string;
  answer: string;
}

export function FAQSection() {
  const [openIndex, setOpenIndex] = useState<number | null>(0);

  const faqs: FAQItem[] = [
    {
      question: 'Do I need administrative privileges to install Listune Studio?',
      answer:
        'No. On Windows, the installer targets your local user profile and registers standard shortcuts without triggering UAC elevation. On macOS, it installs to /Applications or ~/Applications. On Linux, if sudo is not available, it safely installs into ~/.local/share/listune-studio.',
    },
    {
      question: 'Which AI agents are compatible with Listune Studio?',
      answer:
        'Listune Studio connects over the Agent Control Protocol (ACP). It supports Claude Code, opencode, Antigravity CLI, Codex, and any custom agent binary compatible with the standard ACP protocol over stdio.',
    },
    {
      question: 'Does the application work offline?',
      answer:
        'Yes. The application core, database (SQLite), and 28 built-in style previews are entirely local and bundled. Your AI coding agent will only require an internet connection if it calls remote LLM APIs.',
    },
    {
      question: 'Where are rendered videos and projects saved?',
      answer:
        'When you first launch the app, you choose a local working folder. All generated source files, Remotion project folders, briefs, and rendered MP4 files are stored locally in that folder.',
    },
    {
      question: 'How do software updates work?',
      answer:
        'Listune Studio includes an in-app updater for official releases. You can also pin specific versions at any time using the LISTUNE_VERSION environment variable during installation.',
    },
  ];

  const toggle = (index: number) => {
    setOpenIndex(openIndex === index ? null : index);
  };

  return (
    <section id="faq" className="py-12 max-w-4xl mx-auto px-4 sm:px-6">
      <div className="text-center mb-8">
        <h2 className="text-2xl sm:text-3xl font-bold tracking-tight text-white mb-2">
          Frequently Asked Questions
        </h2>
        <p className="text-white/60 text-sm max-w-lg mx-auto">
          Common questions regarding installation, system compatibility, and architecture.
        </p>
      </div>

      <div className="space-y-3">
        {faqs.map((faq, i) => {
          const isOpen = openIndex === i;
          return (
            <GlassCard key={i} variant="subtle" className="overflow-hidden">
              <button
                type="button"
                onClick={() => toggle(i)}
                aria-expanded={isOpen}
                className="w-full p-4 sm:p-5 flex items-center justify-between gap-4 text-left cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#6f9dff]/50"
              >
                <span className="font-medium text-sm sm:text-base text-white">
                  {faq.question}
                </span>
                <ChevronDown
                  className={clsx(
                    'size-4 text-white/50 shrink-0 transition-transform duration-200',
                    isOpen && 'rotate-180 text-white'
                  )}
                />
              </button>
              {isOpen && (
                <div className="px-4 pb-4 sm:px-5 sm:pb-5 text-xs sm:text-sm text-white/70 leading-relaxed border-t border-white/5 pt-3">
                  {faq.answer}
                </div>
              )}
            </GlassCard>
          );
        })}
      </div>
    </section>
  );
}

