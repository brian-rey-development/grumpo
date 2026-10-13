import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';

import { LoadFailure } from '@/components/LoadFailure/LoadFailure';
import { expectAccessible } from '@/test/render';

it('shows what failed with a way to try again, as a page by default', async () => {
  const onRetry = vi.fn();
  const user = userEvent.setup();
  render(
    <LoadFailure
      title="Couldn't load your day"
      description="Try again in a moment."
      retryLabel="Try again"
      onRetry={onRetry}
    />,
  );

  expect(screen.getByRole('alert')).toBeTruthy();
  expect(screen.getByRole('heading', { name: "Couldn't load your day" })).toBeTruthy();
  await user.click(screen.getByRole('button', { name: 'Try again' }));

  expect(onRetry).toHaveBeenCalledOnce();
});

it('sits inside a section without a heading when inline', async () => {
  const { container } = render(
    <LoadFailure
      variant="inline"
      title="Settings couldn't be loaded."
      description="Try again in a moment."
      retryLabel="Try again"
      onRetry={() => undefined}
    />,
  );

  expect(screen.getByRole('alert').textContent).toBe("Settings couldn't be loaded.");
  expect(screen.queryByRole('heading')).toBeNull();
  expect(screen.getByText('Try again in a moment.')).toBeTruthy();
  await expectAccessible(container);
});

it('needs no description', () => {
  render(
    <LoadFailure variant="inline" title="Nope" retryLabel="Retry" onRetry={() => undefined} />,
  );

  expect(screen.getByRole('button', { name: 'Retry' })).toBeTruthy();
});
