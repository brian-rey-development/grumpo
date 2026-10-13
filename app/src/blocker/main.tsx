import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { BlockerApp } from '@/blocker/BlockerApp';
import '@/design/global.css';

const container = document.getElementById('root');
if (!container) {
  throw new Error('Root element #root is missing from blocker.html');
}

createRoot(container).render(
  <StrictMode>
    <BlockerApp />
  </StrictMode>,
);
