import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { Dialog } from '@/components/Dialog/Dialog';
import { useNavigation } from '@/layout/navigation';
import { expectAccessible, renderWithProviders } from '@/test/render';

function Harness({ dismissible = true, onClose }: { dismissible?: boolean; onClose?: () => void }) {
  const [open, setOpen] = useState(false);
  return (
    <>
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
        title="Unlock rules"
        description="Type the sentence to continue"
        dismissible={dismissible}
        onClose={() => {
          onClose?.();
          setOpen(false);
        }}
        footer={<button type="button">Confirm</button>}
      >
        <input aria-label="Phrase" />
      </Dialog>
    </>
  );
}

async function openDialog() {
  const user = userEvent.setup();
  const opener = screen.getByRole('button', { name: 'Open' });
  await user.click(opener);
  return { user, opener };
}

describe('Dialog', () => {
  it('renders an accessible modal and focuses the first control', async () => {
    renderWithProviders(<Harness />);
    await openDialog();

    const dialog = screen.getByRole('dialog', { name: 'Unlock rules' });
    expect(dialog.getAttribute('aria-modal')).toBe('true');
    expect(dialog.getAttribute('aria-describedby')).not.toBeNull();
    expect(document.activeElement).toBe(screen.getByRole('textbox', { name: 'Phrase' }));
  });

  it('keeps Tab and Shift+Tab inside the dialog', async () => {
    renderWithProviders(<Harness />);
    const { user } = await openDialog();
    const input = screen.getByRole('textbox', { name: 'Phrase' });
    const confirm = screen.getByRole('button', { name: 'Confirm' });

    await user.tab();
    expect(document.activeElement).toBe(confirm);
    await user.tab();
    expect(document.activeElement).toBe(input);
    await user.tab({ shift: true });
    expect(document.activeElement).toBe(confirm);
  });

  it('closes on Escape and restores focus to the opener', async () => {
    const onClose = vi.fn();
    renderWithProviders(<Harness onClose={onClose} />);
    const { user, opener } = await openDialog();

    await user.keyboard('{Escape}');

    expect(onClose).toHaveBeenCalledOnce();
    expect(document.activeElement).toBe(opener);
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });

  it('closes when the backdrop is clicked', async () => {
    const onClose = vi.fn();
    renderWithProviders(<Harness onClose={onClose} />);
    const { user } = await openDialog();

    await user.click(screen.getByTestId('dialog-backdrop'));

    expect(onClose).toHaveBeenCalledOnce();
  });

  it('ignores Escape and backdrop clicks when not dismissible', async () => {
    const onClose = vi.fn();
    renderWithProviders(<Harness dismissible={false} onClose={onClose} />);
    const { user } = await openDialog();

    await user.keyboard('{Escape}');
    await user.click(screen.getByTestId('dialog-backdrop'));

    expect(onClose).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog')).toBeTruthy();
  });

  it('pulls focus back when it escapes the dialog', async () => {
    renderWithProviders(<Harness />);
    const { opener } = await openDialog();

    act(() => {
      opener.focus();
    });

    expect(screen.getByRole('dialog').contains(document.activeElement)).toBe(true);
  });

  it('has no accessibility violations while open', async () => {
    renderWithProviders(<Harness />);
    await openDialog();

    await expectAccessible(document.body);
  });
});

describe('app behind a dialog', () => {
  it('is inert while the dialog is open and live again once it closes', async () => {
    const { container } = renderWithProviders(<Harness />);
    const appRoot = container.firstElementChild;
    const { user } = await openDialog();

    expect(appRoot?.hasAttribute('inert')).toBe(true);
    expect(screen.getByRole('dialog').closest('[inert]')).toBeNull();

    await user.keyboard('{Escape}');

    expect(appRoot?.hasAttribute('inert')).toBe(false);
  });

  it('suppresses the Cmd+number screen shortcuts while a dialog is open', async () => {
    renderWithProviders(
      <>
        <ScreenName />
        <Harness />
      </>,
    );
    const { user } = await openDialog();

    await user.keyboard('{Meta>}2{/Meta}');
    expect(screen.getByTestId('screen').textContent).toBe('today');

    await user.keyboard('{Escape}');
    await user.keyboard('{Meta>}2{/Meta}');
    expect(screen.getByTestId('screen').textContent).toBe('week');
  });

  it('fails clearly when a dialog is used without its provider', () => {
    const error = vi.spyOn(console, 'error').mockImplementation(() => undefined);

    expect(() => render(<Dialog open title="x" onClose={() => undefined} />)).toThrow(
      'ModalStackProvider',
    );

    error.mockRestore();
  });
});

function ScreenName() {
  const { screen: current } = useNavigation();
  return <p data-testid="screen">{current}</p>;
}

function Stacked() {
  const [outer, setOuter] = useState(false);
  const [inner, setInner] = useState(false);
  return (
    <>
      <button
        type="button"
        onClick={() => {
          setOuter(true);
        }}
      >
        Open outer
      </button>
      <Dialog
        open={outer}
        title="Outer"
        onClose={() => {
          setOuter(false);
        }}
        footer={
          <button
            type="button"
            onClick={() => {
              setInner(true);
            }}
          >
            Open inner
          </button>
        }
      >
        <input aria-label="Outer field" />
      </Dialog>
      <Dialog
        open={inner}
        title="Inner"
        onClose={() => {
          setInner(false);
        }}
        footer={<button type="button">Inner confirm</button>}
      >
        <input aria-label="Inner field" />
      </Dialog>
    </>
  );
}

describe('stacked dialogs', () => {
  async function openBoth() {
    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: 'Open outer' }));
    await user.click(screen.getByRole('button', { name: 'Open inner' }));
    return user;
  }

  it('lets only the top-most dialog trap focus', async () => {
    renderWithProviders(<Stacked />);
    const user = await openBoth();
    const innerField = screen.getByRole('textbox', { name: 'Inner field' });
    expect(document.activeElement).toBe(innerField);

    await user.tab();
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Inner confirm' }));
    await user.tab();
    expect(document.activeElement).toBe(innerField);
    await user.tab({ shift: true });
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Inner confirm' }));
  });

  it('closes one dialog at a time and gives focus back to the one underneath', async () => {
    renderWithProviders(<Stacked />);
    const user = await openBoth();

    await user.keyboard('{Escape}');

    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'Inner' })).toBeNull();
    });
    expect(screen.getByRole('dialog', { name: 'Outer' })).toBeTruthy();
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Open inner' }));

    await user.tab();
    expect(screen.getByRole('dialog', { name: 'Outer' }).contains(document.activeElement)).toBe(
      true,
    );

    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Open outer' }));
  });

  it('keeps the app inert until the last dialog closes', async () => {
    const { container } = renderWithProviders(<Stacked />);
    const appRoot = container.firstElementChild;
    const user = await openBoth();

    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: 'Inner' })).toBeNull();
    });
    expect(appRoot?.hasAttribute('inert')).toBe(true);

    await user.keyboard('{Escape}');
    expect(appRoot?.hasAttribute('inert')).toBe(false);
  });
});
