import { describe, expect, it } from 'vitest';

import { isSaveShortcut } from '@/features/rules/saveShortcut';

const CMD_S = { key: 's', metaKey: true, ctrlKey: false, altKey: false, shiftKey: false };

describe('isSaveShortcut', () => {
  it('accepts Cmd+S in either case', () => {
    expect(isSaveShortcut(CMD_S)).toBe(true);
    expect(isSaveShortcut({ ...CMD_S, key: 'S' })).toBe(true);
  });

  it.each([
    ['Shift', { shiftKey: true }],
    ['Option', { altKey: true }],
    ['Control', { ctrlKey: true }],
  ])('leaves Cmd+%s+S alone', (_, modifier) => {
    expect(isSaveShortcut({ ...CMD_S, ...modifier })).toBe(false);
  });

  it('needs Cmd and the S key', () => {
    expect(isSaveShortcut({ ...CMD_S, metaKey: false })).toBe(false);
    expect(isSaveShortcut({ ...CMD_S, key: 'a' })).toBe(false);
  });
});
