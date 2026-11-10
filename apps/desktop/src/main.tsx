import React from 'react';
import ReactDOM from 'react-dom/client';
import { TooltipProvider } from '@/components/ui/tooltip';
import App from './App';
import './i18n';
import './styles/tokens.css';

// Disable context menu (inspect) and devtools shortcuts in production builds
if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (e) => {
    e.preventDefault();
  });

  document.addEventListener('keydown', (e) => {
    const isCtrlOrCmd = e.ctrlKey || e.metaKey;
    const key = e.key.toLowerCase();

    if (
      e.key === 'F12' ||
      (isCtrlOrCmd && e.shiftKey && (key === 'i' || key === 'j' || key === 'c')) ||
      (isCtrlOrCmd && key === 'u')
    ) {
      e.preventDefault();
      e.stopPropagation();
    }
  });
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <TooltipProvider>
      <App />
    </TooltipProvider>
  </React.StrictMode>,
);
