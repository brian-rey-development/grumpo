import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { useNavigation } from '@/layout/navigation';
import { renderWithProviders } from '@/test/render';

function CurrentScreen() {
  const { screen: current, navigate } = useNavigation();
  return (
    <>
      <p data-testid="screen">{current}</p>
      <button
        type="button"
        onClick={() => {
          navigate('rules');
        }}
      >
        Go to rules
      </button>
    </>
  );
}

describe('NavigationProvider', () => {
  it('starts on Today and navigates programmatically', async () => {
    const user = userEvent.setup();
    renderWithProviders(<CurrentScreen />);
    expect(screen.getByTestId('screen').textContent).toBe('today');

    await user.click(screen.getByRole('button', { name: 'Go to rules' }));

    expect(screen.getByTestId('screen').textContent).toBe('rules');
  });

  it.each([
    ['1', 'today'],
    ['2', 'week'],
    ['3', 'rules'],
    ['4', 'settings'],
  ])('maps Cmd+%s to the %s screen', (key, expected) => {
    renderWithProviders(<CurrentScreen />);

    fireEvent.keyDown(window, { key: '3', metaKey: true });
    fireEvent.keyDown(window, { key, metaKey: true });

    expect(screen.getByTestId('screen').textContent).toBe(expected);
  });

  it.each([
    ['5', { metaKey: true }],
    ['0', { metaKey: true }],
    ['a', { metaKey: true }],
    ['2', {}],
    ['2', { metaKey: true, shiftKey: true }],
    ['2', { metaKey: true, altKey: true }],
    ['2', { metaKey: true, ctrlKey: true }],
  ])('ignores %s with the modifiers %j', (key, modifiers) => {
    renderWithProviders(<CurrentScreen />);

    fireEvent.keyDown(window, { key, ...modifiers });

    expect(screen.getByTestId('screen').textContent).toBe('today');
  });

  it('throws a clear error when used outside a provider', () => {
    const originalError = console.error;
    console.error = () => undefined;
    try {
      expect(() => render(<CurrentScreen />)).toThrow('NavigationProvider');
    } finally {
      console.error = originalError;
    }
  });
});
