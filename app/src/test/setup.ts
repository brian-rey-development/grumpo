import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

import { installDefaultLocale } from '@/test/locale';

installDefaultLocale('en-GB');

afterEach(() => {
  cleanup();
});

// jsdom has no canvas; axe probes for one and jsdom logs a "not implemented" error otherwise.
HTMLCanvasElement.prototype.getContext = () => null;
