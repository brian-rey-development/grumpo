import { render, screen } from '@testing-library/react';
import { expect, it } from 'vitest';

import { Button } from '@/components/Button/Button';
import { Notice } from '@/components/Notice/Notice';
import { expectAccessible } from '@/test/render';

it('shows a title, text and an action', () => {
  render(
    <Notice tone="warning" title="Locked" role="alert" action={<Button>Start unlock</Button>}>
      These changes weaken focus while it is on.
    </Notice>,
  );

  expect(screen.getByRole('alert').textContent).toContain('Locked');
  expect(screen.getByText('These changes weaken focus while it is on.')).toBeTruthy();
  expect(screen.getByRole('button', { name: 'Start unlock' })).toBeTruthy();
});

it('has no role unless one is given', () => {
  render(<Notice>Rules are read only.</Notice>);

  expect(screen.queryByRole('alert')).toBeNull();
  expect(screen.queryByRole('status')).toBeNull();
});

it.each(['info', 'warning', 'danger'] as const)(
  'has no accessibility violations as %s',
  async (tone) => {
    const { container } = render(
      <Notice tone={tone} title="Title">
        Body
      </Notice>,
    );

    await expectAccessible(container);
  },
);
