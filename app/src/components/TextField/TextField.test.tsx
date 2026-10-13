import { fireEvent, render, screen } from '@testing-library/react';
import { createRef } from 'react';
import { expect, it, vi } from 'vitest';

import { TextField } from '@/components/TextField/TextField';
import { expectAccessible } from '@/test/render';

it('labels the input and links the hint', () => {
  render(<TextField label="Sentence" hint="Type it exactly" />);

  const input = screen.getByLabelText('Sentence');
  const hint = screen.getByText('Type it exactly');
  expect(input.getAttribute('aria-describedby')).toBe(hint.id);
  expect(input.getAttribute('aria-invalid')).toBeNull();
  expect(hint.getAttribute('role')).toBeNull();
});

it('shows the error in place of the hint, marks the input invalid and announces the error', () => {
  render(<TextField label="Sentence" hint="Type it exactly" error="That does not match" />);

  const input = screen.getByLabelText('Sentence');
  expect(input.getAttribute('aria-invalid')).toBe('true');
  const error = screen.getByRole('alert');
  expect(error.textContent).toBe('That does not match');
  expect(error.id).toBe(input.getAttribute('aria-describedby'));
  expect(screen.queryByText('Type it exactly')).toBeNull();
});

it('blocks paste and drop when preventPaste is set', () => {
  const onPaste = vi.fn();
  render(<TextField label="Sentence" preventPaste onPaste={onPaste} />);
  const input = screen.getByLabelText('Sentence');

  const pasteAllowed = fireEvent.paste(input, { clipboardData: { getData: () => 'cheat' } });
  const dropAllowed = fireEvent.drop(input);

  expect(pasteAllowed).toBe(false);
  expect(dropAllowed).toBe(false);
  expect(onPaste).toHaveBeenCalledOnce();
});

it.each(['insertFromPaste', 'insertFromDrop', 'insertReplacementText', 'insertFromYank'])(
  'cancels %s input events when preventPaste is set',
  (inputType) => {
    render(<TextField label="Sentence" preventPaste />);
    const input = screen.getByLabelText('Sentence');
    const allowed = fireEvent(
      input,
      new InputEvent('beforeinput', { inputType, cancelable: true, bubbles: true }),
    );

    expect(allowed).toBe(false);
  },
);

it('lets ordinary typing through when preventPaste is set', () => {
  render(<TextField label="Sentence" preventPaste />);
  const input = screen.getByLabelText('Sentence');
  const event = new InputEvent('beforeinput', {
    inputType: 'insertText',
    data: 'a',
    cancelable: true,
    bubbles: true,
  });

  expect(fireEvent(input, event)).toBe(true);
});

it('turns off spellcheck, autocorrect, autocapitalize and autocomplete when preventPaste is set', () => {
  render(<TextField label="Sentence" preventPaste />);
  const input = screen.getByLabelText('Sentence');

  expect(input.getAttribute('spellcheck')).toBe('false');
  expect(input.getAttribute('autocorrect')).toBe('off');
  expect(input.getAttribute('autocapitalize')).toBe('off');
  expect(input.getAttribute('autocomplete')).toBe('off');
});

it('allows paste and leaves browser assistance alone by default', () => {
  render(<TextField label="Domain" />);
  const input = screen.getByLabelText('Domain');

  expect(fireEvent.paste(input)).toBe(true);
  expect(
    fireEvent(
      input,
      new InputEvent('beforeinput', {
        inputType: 'insertFromPaste',
        cancelable: true,
        bubbles: true,
      }),
    ),
  ).toBe(true);
  expect(input.getAttribute('spellcheck')).toBeNull();
  expect(input.getAttribute('autocomplete')).toBeNull();
});

it('forwards its ref to the input', () => {
  const ref = createRef<HTMLInputElement>();
  render(<TextField label="Domain" ref={ref} />);

  expect(ref.current).toBe(screen.getByLabelText('Domain'));
});

it('has no accessibility violations', async () => {
  const { container } = render(
    <>
      <TextField label="Domain" hint="Without https" />
      <TextField label="Sentence" error="That does not match" preventPaste />
    </>,
  );

  await expectAccessible(container);
});
