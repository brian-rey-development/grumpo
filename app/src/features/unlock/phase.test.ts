import { describe, expect, it } from 'vitest';

import { resolveSheetView, sheetViewKey } from '@/features/unlock/phase';
import { CommandFailure } from '@/ipc/errors';
import { connectedStatus, NOT_INSTALLED } from '@/test/ipc';

describe('resolveSheetView', () => {
  it('is loading until the first status arrives', () => {
    expect(resolveSheetView(undefined, null)).toEqual({ kind: 'loading' });
  });

  it('fails when the first read failed', () => {
    const failure = new CommandFailure('internal', 'No answer');
    const view = resolveSheetView(undefined, failure);
    expect(view).toEqual({ kind: 'failed', failure });
  });

  it('keeps the last status when a later refresh failed', () => {
    const view = resolveSheetView(connectedStatus(), new CommandFailure('internal', 'No answer'));
    expect(view.kind).toBe('phase');
  });

  it('reports why the service is unavailable', () => {
    expect(resolveSheetView(NOT_INSTALLED, null)).toEqual({
      kind: 'unavailable',
      reason: 'notInstalled',
    });
  });

  it.each(['locked', 'coolingDown', 'ready', 'unlocked'] as const)('keys the %s phase', (kind) => {
    const phases = {
      locked: { kind: 'locked' },
      coolingDown: { kind: 'coolingDown', readyAt: 1, minutes: 5 },
      ready: { kind: 'ready', readyAt: 1, expiresAt: 2 },
      unlocked: { kind: 'unlocked', until: 3 },
    } as const;
    const view = resolveSheetView(connectedStatus({ phase: phases[kind] }), null);
    expect(sheetViewKey(view)).toBe(kind);
  });

  it('keys the fallback views by their own name', () => {
    expect(sheetViewKey({ kind: 'loading' })).toBe('loading');
    expect(sheetViewKey({ kind: 'unavailable', reason: 'notResponding' })).toBe('unavailable');
  });
});
