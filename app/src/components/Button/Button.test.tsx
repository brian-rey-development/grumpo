import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';

import { Button } from '@/components/Button/Button';

it('defaults to type button so it never submits a form by accident', () => {
  render(<Button>Save</Button>);

  expect(screen.getByRole('button', { name: 'Save' }).getAttribute('type')).toBe('button');
});

it('stays focusable and ignores clicks while loading', async () => {
  const onClick = vi.fn();
  const user = userEvent.setup();
  render(
    <Button loading onClick={onClick}>
      Save
    </Button>,
  );
  const button = screen.getByRole('button', { name: 'Save' });

  await user.click(button);

  expect(button.hasAttribute('disabled')).toBe(false);
  expect(button.getAttribute('aria-disabled')).toBe('true');
  expect(button.getAttribute('aria-busy')).toBe('true');
  expect(document.activeElement).toBe(button);
  expect(onClick).not.toHaveBeenCalled();
});

it('keeps focus on a button that starts loading after being pressed', async () => {
  const user = userEvent.setup();
  const { rerender } = render(<Button>Save</Button>);
  const button = screen.getByRole('button', { name: 'Save' });

  await user.click(button);
  rerender(<Button loading>Save</Button>);

  expect(document.activeElement).toBe(button);
});

it('does not submit a form while loading', async () => {
  const onSubmit = vi.fn((event: { preventDefault: () => void }) => {
    event.preventDefault();
  });
  const user = userEvent.setup();
  render(
    <form onSubmit={onSubmit}>
      <Button type="submit" loading>
        Save
      </Button>
    </form>,
  );

  await user.click(screen.getByRole('button', { name: 'Save' }));

  expect(onSubmit).not.toHaveBeenCalled();
});

it('is really disabled when asked to be', async () => {
  const onClick = vi.fn();
  const user = userEvent.setup();
  render(
    <Button disabled onClick={onClick}>
      Save
    </Button>,
  );

  await user.click(screen.getByRole('button', { name: 'Save' }));

  expect(onClick).not.toHaveBeenCalled();
});
