import { screen, within } from '@testing-library/react';
import { expect, it } from 'vitest';

import { BarList } from '@/components/BarList/BarList';
import { expectAccessible, renderWithProviders } from '@/test/render';

function rowAt(rows: HTMLElement[], index: number): HTMLElement {
  const row = rows[index];
  if (row === undefined) throw new Error(`Missing row ${index.toString()}`);
  return row;
}

const EMPTY = 'Nothing tracked yet';

const ITEMS = [
  { key: 'com.apple.Safari', label: 'Safari', seconds: 15_120 },
  { key: 'com.apple.dt.Xcode', label: 'Xcode', seconds: 2_280 },
  { key: 'github.com', label: 'github.com', seconds: 30 },
];

it('renders a labelled list with readable durations for every row', () => {
  renderWithProviders(<BarList label="Top apps" items={ITEMS} emptyText={EMPTY} />);

  const list = screen.getByRole('list', { name: 'Top apps' });
  const rows = within(list).getAllByRole('listitem');
  expect(rows).toHaveLength(3);
  expect(within(rowAt(rows, 0)).getByText('Safari')).toBeTruthy();
  expect(within(rowAt(rows, 0)).getByText('4h 12min')).toBeTruthy();
  expect(within(rowAt(rows, 1)).getByText('38min')).toBeTruthy();
  expect(within(rowAt(rows, 2)).getByText('< 1min')).toBeTruthy();
});

it('hides the decorative bars from assistive technology', () => {
  const { container } = renderWithProviders(
    <BarList label="Top apps" items={ITEMS} emptyText={EMPTY} />,
  );

  expect(container.querySelectorAll('[aria-hidden="true"]').length).toBeGreaterThanOrEqual(
    ITEMS.length,
  );
});

it('shows the leading visual when provided', () => {
  renderWithProviders(
    <BarList
      label="Top apps"
      items={[{ key: 'a', label: 'A', seconds: 60, leading: <i data-testid="icon" /> }]}
      emptyText={EMPTY}
    />,
  );

  expect(screen.getByTestId('icon')).toBeTruthy();
});

it('shows an empty message instead of an empty list', () => {
  renderWithProviders(<BarList label="Top sites" items={[]} emptyText="No sites visited" />);

  expect(screen.queryByRole('list')).toBeNull();
  expect(screen.getByText('No sites visited')).toBeTruthy();
});

it('shows a daily limit as a meter with the words under it', () => {
  renderWithProviders(
    <BarList
      label="Top sites"
      items={[
        {
          key: 'youtube.com',
          label: 'youtube.com',
          seconds: 1_800,
          limit: {
            fraction: 0.75,
            level: 'ok',
            note: '8 min left of 30 min',
            spoken: '22 min of 30 min',
          },
        },
      ]}
      emptyText={EMPTY}
    />,
  );

  const meter = screen.getByRole('progressbar', { name: 'youtube.com' });
  expect(meter.getAttribute('aria-valuenow')).toBe('75');
  expect(meter.getAttribute('aria-valuetext')).toBe('22 min of 30 min');
  expect(screen.getByText('8 min left of 30 min')).toBeTruthy();
});

it('has no accessibility violations', async () => {
  const { container } = renderWithProviders(
    <BarList label="Top apps" items={ITEMS} emptyText={EMPTY} />,
  );

  await expectAccessible(container);
});
