import { useTranslation } from 'react-i18next';
import type { ProjectStatus } from '@listune/shared';
import { cn } from '@/lib/utils';

export const STEPS = ['topic', 'script', 'style', 'build', 'render'] as const;

/**
 * Index of the step the project is on. Topic is always done. draft → Script; building → Build (Script and
 * Style done); a render running for the project → Render; rendered → 5: every step done, none current.
 * A rendered video that renders again is on Render.
 */
export function currentStep(status: ProjectStatus, rendering = false): number {
  if (rendering) return 4;
  if (status === 'draft') return 1;
  if (status === 'building') return 3;
  return STEPS.length;
}

interface Props {
  status: ProjectStatus;
  /** A render of this project is running (state/render.ts). */
  rendering?: boolean;
}

/** A project's progress (`status`). */
export function StepBar({ status, rendering = false }: Props) {
  const { t } = useTranslation();
  const names = STEPS.map((s) => t(`stepBar.${s}`));
  const cur = currentStep(status, rendering);
  return (
    // A project's steps sit in the chat header: one line always, and when its container (`@container/steps`)
    // is narrow, only the current step keeps its name.
    <ol
      aria-label={t('stepBar.aria')}
      className="m-0 flex list-none flex-nowrap items-center gap-2 p-0"
    >
      {names.map((name, i) => {
        const done = i < cur;
        const active = i === cur;
        const pulse = active && rendering;
        return (
          <li key={name} title={name} aria-current={active ? 'step' : undefined} className="flex shrink-0 items-center gap-2">
            <span
              aria-hidden="true"
              data-step-mark
              className={cn(
                'inline-flex size-5 items-center justify-center rounded-full font-mono text-[11px]',
                done && 'bg-deep-rose text-rose-light',
                active && 'bg-primary text-primary-foreground',
                !done && !active && 'border border-hairline text-text-3',
              )}
            >
              {done ? '✓' : i + 1}
            </span>
            <span
              data-step-name
              className={cn(
                'text-[13px] whitespace-nowrap',
                active ? 'font-medium text-foreground' : 'text-text-2',
                !active && '@max-[34rem]/steps:sr-only',
              )}
            >
              {name}
            </span>
            {pulse && (
              <>
                {/* Still under reduced motion. */}
                <span aria-hidden="true" data-step-pulse className="size-1.5 rounded-full bg-primary motion-safe:animate-pulse" />
                <span className="sr-only">{t('stepBar.renderingNow')}</span>
              </>
            )}
            {i < names.length - 1 && <span aria-hidden="true" className="h-px w-4 shrink-0 bg-hairline @max-[34rem]/steps:w-2" />}
          </li>
        );
      })}
    </ol>
  );
}
