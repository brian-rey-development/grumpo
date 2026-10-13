import { render, screen, waitFor } from '@testing-library/react';
import { expect, it } from 'vitest';

import { PhaseStage } from '@/features/unlock/PhaseStage';

it('sizes itself to its content when the browser cannot measure', () => {
  render(
    <PhaseStage phaseKey="a">
      <p>First</p>
    </PhaseStage>,
  );

  expect(screen.getByText('First')).toBeTruthy();
});

it('keeps the same content mounted while the phase key stays put', () => {
  const { rerender } = render(
    <PhaseStage phaseKey="a">
      <p>Count 1</p>
    </PhaseStage>,
  );
  const before = screen.getByText('Count 1');

  rerender(
    <PhaseStage phaseKey="a">
      <p>Count 2</p>
    </PhaseStage>,
  );

  expect(screen.getByText('Count 2').parentElement).toBe(before.parentElement);
});

it('swaps the content when the phase key changes and focuses the marked control', async () => {
  const { rerender } = render(
    <PhaseStage phaseKey="a">
      <button type="button">First</button>
    </PhaseStage>,
  );

  rerender(
    <PhaseStage phaseKey="b">
      <button type="button">Plain</button>
      <button type="button" data-autofocus>
        Main
      </button>
    </PhaseStage>,
  );

  const main = await screen.findByRole('button', { name: 'Main' });
  await waitFor(() => {
    expect(document.activeElement).toBe(main);
  });
  expect(screen.queryByRole('button', { name: 'First' })).toBeNull();
});
