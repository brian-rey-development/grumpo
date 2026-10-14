import { describe, expect, it } from 'vitest';

import { hostsOf } from '@/features/rules/hosts';
import { HOSTS } from '@/test/vectors';

describe('hostsOf', () => {
  it.each(HOSTS)('writes %s as the service does', (site, hosts) => {
    expect(hostsOf(site).toSorted()).toEqual(hosts.toSorted());
  });
});
