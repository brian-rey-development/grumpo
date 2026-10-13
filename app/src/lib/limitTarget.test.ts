import { describe, expect, it } from 'vitest';

import { targetKey, targetName } from '@/lib/limitTarget';
import { YOUTUBE } from '@/test/ipc';

const SAFARI = { kind: 'app', app: { bundleId: 'com.apple.Safari', name: 'Safari' } } as const;

describe('limit targets', () => {
  it('names a site by its domain and an app by its display name', () => {
    expect(targetName(YOUTUBE)).toBe('youtube.com');
    expect(targetName(SAFARI)).toBe('Safari');
  });

  it('keys sites and apps apart', () => {
    expect(targetKey(YOUTUBE)).toBe('site:youtube.com');
    expect(targetKey(SAFARI)).toBe('app:com.apple.Safari');
  });

  it('never gives a site and an app the same key', () => {
    const site = { kind: 'site', domain: 'com.apple.Safari' } as const;
    expect(targetKey(site)).not.toBe(targetKey(SAFARI));
  });
});
