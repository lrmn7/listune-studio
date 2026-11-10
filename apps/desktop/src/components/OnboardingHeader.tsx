import { useTranslation } from 'react-i18next';
import logo from '@/assets/brand/listune-logo.png';
import { cn } from '@/lib/utils';

/** Top bar shared by the onboarding screens. The bar itself drags the window. */
/** Onboarding has three steps: Welcome, Connect, and Requirements. */
export const ONBOARDING_STEPS = 3;

export function OnboardingHeader({ step, bars }: { step: 1 | 2 | 3; bars?: boolean }) {
  const { t } = useTranslation();
  return (
    <div data-tauri-drag-region className="flex items-center justify-between">
      <div className="flex items-center gap-2">
        <img src={logo} alt="" aria-hidden="true" className="block size-8 shrink-0" />
        <span className="text-[17px] font-bold tracking-[-0.02em] text-foreground">Listune</span>
        <span className="mt-0.5 text-[13px] text-text-3">{t('sidebar.studio')}</span>
      </div>
      <div className="flex items-center gap-3">
        {bars && (
          <div aria-hidden="true" className="flex gap-1">
            {Array.from({ length: ONBOARDING_STEPS }, (_, i) => i + 1).map((n) => (
              <div key={n} className={cn('h-1 w-6 rounded-full', n <= step ? 'bg-foreground' : 'bg-hairline')} />
            ))}
          </div>
        )}
        <span className="text-[13px] text-text-3">
          {t('onboarding.step', { step, total: ONBOARDING_STEPS })}
        </span>
      </div>
    </div>
  );
}
