import { act, renderHook } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { addSite, removeSite } from '@/features/rules/rulesModel';
import { useRulesDraft } from '@/features/rules/useRulesDraft';
import type { Rules } from '@/ipc/generated/Rules';
import { EMPTY_RULES, SAMPLE_RULES } from '@/test/ipc';

function setup(initial: Rules | null) {
  return renderHook(({ connected }: { connected: Rules | null }) => useRulesDraft(connected), {
    initialProps: { connected: initial },
  });
}

describe('useRulesDraft', () => {
  it('starts as the saved rules with nothing to save', () => {
    const { result } = setup(SAMPLE_RULES);

    expect(result.current.rules).toBe(SAMPLE_RULES);
    expect(result.current.saved).toBe(SAMPLE_RULES);
    expect(result.current.dirty).toBe(false);
  });

  it('has no rules before the service has ever answered', () => {
    const { result } = setup(null);

    expect(result.current.rules).toBeNull();
    act(() => {
      result.current.edit((rules) => addSite(rules, 'x.com'));
    });
    expect(result.current.rules).toBeNull();
    expect(result.current.dirty).toBe(false);
  });

  it('keeps an edit apart from the saved rules and marks it dirty', () => {
    const { result } = setup(SAMPLE_RULES);

    act(() => {
      result.current.edit((rules) => addSite(rules, 'news.com'));
    });

    expect(result.current.rules?.blockedSites).toContain('news.com');
    expect(result.current.saved).toBe(SAMPLE_RULES);
    expect(result.current.dirty).toBe(true);
  });

  it('stacks edits on the latest draft', () => {
    const { result } = setup(SAMPLE_RULES);

    act(() => {
      result.current.edit((rules) => addSite(rules, 'a.com'));
      result.current.edit((rules) => addSite(rules, 'b.com'));
    });

    expect(result.current.rules?.blockedSites).toEqual([
      'a.com',
      'b.com',
      'reddit.com',
      'youtube.com',
    ]);
  });

  it('is clean again when an edit is undone by hand', () => {
    const { result } = setup(SAMPLE_RULES);

    act(() => {
      result.current.edit((rules) => addSite(rules, 'a.com'));
    });
    act(() => {
      result.current.edit((rules) => removeSite(rules, 'a.com'));
    });

    expect(result.current.dirty).toBe(false);
  });

  it('discards back to the saved rules', () => {
    const { result } = setup(SAMPLE_RULES);
    act(() => {
      result.current.edit((rules) => removeSite(rules, 'reddit.com'));
    });

    act(() => {
      result.current.discard();
    });

    expect(result.current.rules).toBe(SAMPLE_RULES);
    expect(result.current.dirty).toBe(false);
  });

  it('follows the saved rules while there are no edits', () => {
    const { result, rerender } = setup(SAMPLE_RULES);

    rerender({ connected: EMPTY_RULES });

    expect(result.current.rules).toBe(EMPTY_RULES);
    expect(result.current.dirty).toBe(false);
  });

  it('keeps the draft when the saved rules change underneath it', () => {
    const { result, rerender } = setup(SAMPLE_RULES);
    act(() => {
      result.current.edit((rules) => addSite(rules, 'mine.com'));
    });

    rerender({ connected: EMPTY_RULES });

    expect(result.current.rules?.blockedSites).toContain('mine.com');
    expect(result.current.saved).toBe(EMPTY_RULES);
    expect(result.current.dirty).toBe(true);
  });

  it('is clean when the service ends up holding exactly the draft', () => {
    const { result, rerender } = setup(SAMPLE_RULES);
    act(() => {
      result.current.edit((rules) => removeSite(rules, 'reddit.com'));
    });

    rerender({ connected: removeSite(SAMPLE_RULES, 'reddit.com') });

    expect(result.current.dirty).toBe(false);
  });

  it('remembers the last saved rules when the service goes away', () => {
    const { result, rerender } = setup(SAMPLE_RULES);

    rerender({ connected: null });

    expect(result.current.rules).toBe(SAMPLE_RULES);
    expect(result.current.saved).toBe(SAMPLE_RULES);
    expect(result.current.dirty).toBe(false);
  });

  it('is read only exactly while the service is away', () => {
    const { result, rerender } = setup(SAMPLE_RULES);
    expect(result.current.readOnly).toBe(false);

    rerender({ connected: null });
    expect(result.current.readOnly).toBe(true);

    rerender({ connected: SAMPLE_RULES });
    expect(result.current.readOnly).toBe(false);
  });

  it('keeps the edits made before the service went away', () => {
    const { result, rerender } = setup(SAMPLE_RULES);
    act(() => {
      result.current.edit((rules) => addSite(rules, 'mine.com'));
    });

    rerender({ connected: null });

    expect(result.current.dirty).toBe(true);
    expect(result.current.rules?.blockedSites).toContain('mine.com');
  });
});
