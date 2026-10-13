import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from '@/App';
import { watchWindowFocus } from '@/ipc/windowFocus';
import '@/design/global.css';

watchWindowFocus();

const container = document.getElementById('root');
if (!container) {
  throw new Error('Root element #root is missing from index.html');
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
