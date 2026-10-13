import { render, screen, within } from '@testing-library/react';
import { expect, it } from 'vitest';

import { Section } from '@/components/Section/Section';
import { SectionRow } from '@/components/Section/SectionRow';
import { expectAccessible } from '@/test/render';

it('renders free-form content in a labelled group rather than a list', () => {
  render(
    <Section title="Activity" description="Updated a minute ago">
      <p>Chart goes here</p>
    </Section>,
  );

  const group = screen.getByRole('group', { name: 'Activity' });
  expect(within(group).getByText('Chart goes here')).toBeTruthy();
  expect(screen.queryByRole('list')).toBeNull();
  expect(screen.getByRole('heading', { level: 2, name: 'Activity' })).toBeTruthy();
});

it('renders rows in a real labelled list when asked to', () => {
  render(
    <Section title="Blocked sites" as="list">
      <SectionRow label="youtube.com" detail="Blocked on weekdays" />
      <SectionRow label="x.com" />
    </Section>,
  );

  const list = screen.getByRole('list', { name: 'Blocked sites' });
  expect(within(list).getAllByRole('listitem')).toHaveLength(2);
  expect(screen.queryByRole('group')).toBeNull();
});

it('does not invent a group role for untitled content', () => {
  render(
    <Section>
      <p>Loose content</p>
    </Section>,
  );

  expect(screen.queryByRole('group')).toBeNull();
});

it('shows the header action next to the title', () => {
  render(
    <Section title="Limits" action={<button type="button">Add limit</button>}>
      <p>None yet</p>
    </Section>,
  );

  expect(screen.getByRole('button', { name: 'Add limit' })).toBeTruthy();
});

it('has no accessibility violations in either form', async () => {
  const { container } = render(
    <>
      <Section title="Activity">
        <p>Chart</p>
      </Section>
      <Section title="Sites" as="list">
        <SectionRow label="youtube.com" />
      </Section>
    </>,
  );

  await expectAccessible(container);
});
