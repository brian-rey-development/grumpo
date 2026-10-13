import { render, screen } from '@testing-library/react';
import { expect, it } from 'vitest';

import { SiteBadge } from '@/components/SiteBadge/SiteBadge';

it('marks a site with a globe and no letter', () => {
  const { container } = render(<SiteBadge />);

  expect(container.querySelector('svg')).not.toBeNull();
  expect(container.textContent).toBe('');
});

it('hides the globe from assistive technology', () => {
  render(<SiteBadge />);

  expect(screen.getByTestId('site-badge').getAttribute('aria-hidden')).toBe('true');
});
