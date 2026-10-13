import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, type RenderResult } from '@testing-library/react';
import type { ReactElement, ReactNode } from 'react';
import { axe, configureAxe } from 'vitest-axe';
import { expect } from 'vitest';

import { ModalStackProvider } from '@/components/Dialog/ModalStackProvider';
import { RevealProvider } from '@/components/Reveal/Reveal';
import { createQueryClient } from '@/ipc/queryClient';
import { NavigationProvider } from '@/layout/NavigationProvider';
import type { Screen } from '@/layout/navigation';

export function createTestQueryClient(): QueryClient {
  const client = createQueryClient();
  client.setDefaultOptions({
    queries: { ...client.getDefaultOptions().queries, retry: false, gcTime: Infinity },
    mutations: { retry: false },
  });
  return client;
}

interface ProviderOptions {
  client?: QueryClient;
  screen?: Screen;
}

export type ProviderRenderResult = RenderResult & { client: QueryClient };

export function renderWithProviders(
  ui: ReactElement,
  { client = createTestQueryClient(), screen = 'today' }: ProviderOptions = {},
): ProviderRenderResult {
  function Wrapper({ children }: { children: ReactNode }) {
    return (
      <QueryClientProvider client={client}>
        <RevealProvider>
          <ModalStackProvider>
            <NavigationProvider initialScreen={screen}>{children}</NavigationProvider>
          </ModalStackProvider>
        </RevealProvider>
      </QueryClientProvider>
    );
  }
  return Object.assign(render(ui, { wrapper: Wrapper }), { client });
}

const componentAxe = configureAxe({ rules: { region: { enabled: false } } });

export async function expectAccessible(container: Element, landmarks = false): Promise<void> {
  const results = await (landmarks ? axe(container) : componentAxe(container));
  const violations = results.violations.map((violation) => `${violation.id}: ${violation.help}`);
  expect(violations).toEqual([]);
}
