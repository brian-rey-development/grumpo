import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { expect, it, vi } from 'vitest';

import { Toggle } from '@/components/Toggle/Toggle';
import { expectAccessible } from '@/test/render';

function Controlled({ onChange }: { onChange?: (checked: boolean) => void }) {
  const [checked, setChecked] = useState(false);
  return (
    <Toggle
      label="Launch at login"
      checked={checked}
      onChange={(next) => {
        onChange?.(next);
        setChecked(next);
      }}
    />
  );
}

it('exposes a labelled switch with its state', () => {
  render(<Controlled />);

  const toggle = screen.getByRole('switch', { name: 'Launch at login' });
  expect(toggle.getAttribute('aria-checked')).toBe('false');
});

it('toggles with Space and Enter from the keyboard', async () => {
  const onChange = vi.fn();
  const user = userEvent.setup();
  render(<Controlled onChange={onChange} />);

  await user.tab();
  const toggle = screen.getByRole('switch');
  expect(document.activeElement).toBe(toggle);

  await user.keyboard(' ');
  expect(toggle.getAttribute('aria-checked')).toBe('true');
  await user.keyboard('{Enter}');
  expect(toggle.getAttribute('aria-checked')).toBe('false');
  expect(onChange.mock.calls).toEqual([[true], [false]]);
});

it('does not change when disabled', async () => {
  const onChange = vi.fn();
  const user = userEvent.setup();
  render(<Toggle label="Locked" checked={false} onChange={onChange} disabled />);

  await user.click(screen.getByRole('switch'));

  expect(onChange).not.toHaveBeenCalled();
});

it('has no accessibility violations', async () => {
  const { container } = render(
    <>
      <Toggle label="Launch at login" checked={false} onChange={() => undefined} />
      <Toggle label="Block sites" checked onChange={() => undefined} />
    </>,
  );

  await expectAccessible(container);
});
