import { fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { type ComponentProps, useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { DateStepper } from '@/components/DateStepper/DateStepper';
import { Dialog } from '@/components/Dialog/Dialog';
import { expectAccessible, renderWithProviders } from '@/test/render';

type DateStepperProps = ComponentProps<typeof DateStepper>;

function props(overrides: Partial<DateStepperProps> = {}): DateStepperProps {
  return {
    label: 'Choose day',
    previousLabel: 'Previous day',
    nextLabel: 'Next day',
    canGoNext: true,
    onPrevious: vi.fn(),
    onNext: vi.fn(),
    reset: null,
    ...overrides,
  };
}

describe('DateStepper', () => {
  it('steps with its buttons, which are reachable by keyboard', async () => {
    const user = userEvent.setup();
    const config = props();
    renderWithProviders(<DateStepper {...config} />);

    await user.tab();
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Previous day' }));
    await user.keyboard('{Enter}');
    await user.tab();
    await user.keyboard(' ');

    expect(config.onPrevious).toHaveBeenCalledTimes(1);
    expect(config.onNext).toHaveBeenCalledTimes(1);
  });

  it('marks the next button unavailable without dropping it from the tab order', async () => {
    const config = props({ canGoNext: false });
    const user = userEvent.setup();
    renderWithProviders(<DateStepper {...config} />);
    const next = screen.getByRole('button', { name: 'Next day' });

    await user.click(next);

    expect(next.getAttribute('aria-disabled')).toBe('true');
    expect(config.onNext).not.toHaveBeenCalled();
    expect(next.tabIndex).toBe(0);
  });

  it('keeps focus on the stepper when Today brings the person back to the present', async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    const { rerender } = renderWithProviders(
      <DateStepper {...props({ reset: { label: 'Today', onSelect } })} />,
    );

    await user.click(screen.getByRole('button', { name: 'Today' }));
    rerender(<DateStepper {...props()} />);

    expect(onSelect).toHaveBeenCalledTimes(1);
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Previous day' }));
  });

  it('keeps focus on Next when stepping reaches the present', async () => {
    const user = userEvent.setup();
    const { rerender } = renderWithProviders(
      <DateStepper {...props({ reset: { label: 'Today', onSelect: vi.fn() } })} />,
    );
    const next = screen.getByRole('button', { name: 'Next day' });

    await user.click(next);
    rerender(<DateStepper {...props({ canGoNext: false })} />);

    expect(document.activeElement).toBe(next);
  });

  it('offers a way back only when away from the present', async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    const { rerender } = renderWithProviders(<DateStepper {...props()} />);
    expect(screen.queryByRole('button', { name: 'Today' })).toBeNull();

    rerender(<DateStepper {...props({ reset: { label: 'Today', onSelect } })} />);
    await user.click(screen.getByRole('button', { name: 'Today' }));

    expect(onSelect).toHaveBeenCalledTimes(1);
  });

  it('steps with Cmd+[ and Cmd+] and ignores Cmd+] when the next button is disabled', () => {
    const config = props();
    const { rerender } = renderWithProviders(<DateStepper {...config} />);

    fireEvent.keyDown(window, { key: '[', metaKey: true });
    fireEvent.keyDown(window, { key: ']', metaKey: true });
    expect(config.onPrevious).toHaveBeenCalledTimes(1);
    expect(config.onNext).toHaveBeenCalledTimes(1);

    rerender(<DateStepper {...config} canGoNext={false} />);
    fireEvent.keyDown(window, { key: ']', metaKey: true });

    expect(config.onNext).toHaveBeenCalledTimes(1);
  });

  it.each([
    ['a', {}],
    ['[', {}],
    ['[', { metaKey: true, shiftKey: true }],
    ['[', { metaKey: true, altKey: true }],
    ['[', { metaKey: true, ctrlKey: true }],
  ])('ignores %s with the modifiers %j', (key, modifiers) => {
    const config = props();
    renderWithProviders(<DateStepper {...config} />);

    fireEvent.keyDown(window, { key, ...modifiers });

    expect(config.onPrevious).not.toHaveBeenCalled();
    expect(config.onNext).not.toHaveBeenCalled();
  });

  it('stops listening for shortcuts while a dialog is open and resumes after', async () => {
    const config = props();
    function Harness() {
      const [open, setOpen] = useState(false);
      return (
        <>
          <DateStepper {...config} />
          <button
            type="button"
            onClick={() => {
              setOpen(true);
            }}
          >
            Open
          </button>
          <Dialog
            open={open}
            title="Unlock"
            onClose={() => {
              setOpen(false);
            }}
          >
            <input aria-label="Phrase" />
          </Dialog>
        </>
      );
    }
    const user = userEvent.setup();
    renderWithProviders(<Harness />);

    await user.click(screen.getByRole('button', { name: 'Open' }));
    fireEvent.keyDown(window, { key: '[', metaKey: true });
    expect(config.onPrevious).not.toHaveBeenCalled();

    await user.keyboard('{Escape}');
    await vi.waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    fireEvent.keyDown(window, { key: '[', metaKey: true });

    expect(config.onPrevious).toHaveBeenCalledTimes(1);
  });

  it('disables Previous and its shortcut when there is no older history', async () => {
    const config = props({ canGoPrevious: false });
    const user = userEvent.setup();
    renderWithProviders(<DateStepper {...config} />);
    const previous = screen.getByRole('button', { name: 'Previous day' });

    await user.click(previous);
    fireEvent.keyDown(window, { key: '[', metaKey: true });

    expect(previous.getAttribute('aria-disabled')).toBe('true');
    expect(config.onPrevious).not.toHaveBeenCalled();
  });

  it('keeps Previous available unless told otherwise', () => {
    renderWithProviders(<DateStepper {...props()} />);

    expect(screen.getByRole('button', { name: 'Previous day' }).hasAttribute('aria-disabled')).toBe(
      false,
    );
  });

  it('removes its shortcuts when it unmounts', () => {
    const config = props();
    const { unmount } = renderWithProviders(<DateStepper {...config} />);

    unmount();
    fireEvent.keyDown(window, { key: '[', metaKey: true });

    expect(config.onPrevious).not.toHaveBeenCalled();
  });

  it('has no accessibility violations', async () => {
    const { container } = renderWithProviders(
      <DateStepper {...props({ reset: { label: 'Today', onSelect: vi.fn() } })} />,
    );

    await expectAccessible(container);
  });
});
