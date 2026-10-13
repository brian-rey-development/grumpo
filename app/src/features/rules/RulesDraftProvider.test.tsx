import { QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { RulesDraftProvider } from '@/features/rules/RulesDraftProvider';
import { useRulesDirty, useRulesDraftContext } from '@/features/rules/rulesDraftContext';
import { addSite } from '@/features/rules/rulesModel';
import { queryKeys } from '@/ipc/queryKeys';
import { connectedStatus, NOT_INSTALLED, routeCommands, SAMPLE_RULES } from '@/test/ipc';
import { createTestQueryClient } from '@/test/render';

beforeEach(() => {
  invokeMock.mockReset();
  routeCommands(invokeMock, { get_focus_status: () => connectedStatus({ active: false }) });
});

function setup() {
  const client = createTestQueryClient();
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>
      <RulesDraftProvider>{children}</RulesDraftProvider>
    </QueryClientProvider>
  );
  const view = renderHook(() => ({ draft: useRulesDraftContext(), dirty: useRulesDirty() }), {
    wrapper,
  });
  return { client, ...view };
}

describe('RulesDraftProvider', () => {
  it('serves the rules the service holds, with no edits to begin with', async () => {
    const { result } = setup();

    await waitFor(() => {
      expect(result.current.draft.rules).toEqual(SAMPLE_RULES);
    });
    expect(result.current.dirty).toBe(false);
    expect(result.current.draft.readOnly).toBe(false);
  });

  it('holds an edit and reports it as unsaved to anyone who asks', async () => {
    const { result } = setup();
    await waitFor(() => {
      expect(result.current.draft.rules).not.toBeNull();
    });

    act(() => {
      result.current.draft.edit((rules) => addSite(rules, 'news.com'));
    });

    expect(result.current.dirty).toBe(true);
    expect(result.current.draft.rules?.blockedSites).toContain('news.com');
  });

  it('keeps the edit while the service goes away and comes back', async () => {
    const { result, client } = setup();
    await waitFor(() => {
      expect(result.current.draft.rules).not.toBeNull();
    });
    act(() => {
      result.current.draft.edit((rules) => addSite(rules, 'news.com'));
    });

    act(() => {
      client.setQueryData(queryKeys.focusStatus, NOT_INSTALLED);
    });
    await waitFor(() => {
      expect(result.current.draft.readOnly).toBe(true);
    });
    expect(result.current.dirty).toBe(true);

    act(() => {
      client.setQueryData(queryKeys.focusStatus, connectedStatus({ active: false }));
    });
    await waitFor(() => {
      expect(result.current.draft.readOnly).toBe(false);
    });
    expect(result.current.draft.rules?.blockedSites).toContain('news.com');
  });

  it('is clean again after the edits are discarded', async () => {
    const { result } = setup();
    await waitFor(() => {
      expect(result.current.draft.rules).not.toBeNull();
    });
    act(() => {
      result.current.draft.edit((rules) => addSite(rules, 'news.com'));
    });

    act(() => {
      result.current.draft.discard();
    });

    expect(result.current.dirty).toBe(false);
  });
});

describe('outside a provider', () => {
  it('refuses to hand out a draft', () => {
    const silence = vi.spyOn(console, 'error').mockImplementation(() => undefined);

    expect(() => renderHook(() => useRulesDraftContext())).toThrow(/RulesDraftProvider/);
    silence.mockRestore();
  });

  it('refuses to report the unsaved state too', () => {
    const silence = vi.spyOn(console, 'error').mockImplementation(() => undefined);

    expect(() => renderHook(() => useRulesDirty())).toThrow(/RulesDraftProvider/);
    silence.mockRestore();
  });
});
