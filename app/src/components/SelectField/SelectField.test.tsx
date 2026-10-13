import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { SelectField } from '@/components/SelectField/SelectField';
import { expectAccessible } from '@/test/render';

const OPTIONS = [
  { value: '30', label: '30 days' },
  { value: '90', label: '90 days' },
  { value: '365', label: '1 year' },
] as const;

describe('SelectField', () => {
  it('is the system pop-up menu with the chosen option selected', () => {
    render(
      <SelectField label="Keep history for" options={OPTIONS} value="90" onChange={vi.fn()} />,
    );

    const select = screen.getByRole('combobox', { name: 'Keep history for' });
    expect(select.tagName).toBe('SELECT');
    expect(select instanceof HTMLSelectElement && select.value).toBe('90');
    expect(screen.getAllByRole('option').map((option) => option.textContent)).toEqual([
      '30 days',
      '90 days',
      '1 year',
    ]);
  });

  it('reports the value of the option picked', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(
      <SelectField label="Keep history for" options={OPTIONS} value="90" onChange={onChange} />,
    );

    await user.selectOptions(screen.getByRole('combobox'), '365');

    expect(onChange).toHaveBeenCalledExactlyOnceWith('365');
  });

  it('follows the value it is given rather than keeping its own', async () => {
    const user = userEvent.setup();
    render(
      <SelectField label="Keep history for" options={OPTIONS} value="90" onChange={vi.fn()} />,
    );

    await user.selectOptions(screen.getByRole('combobox'), '30');

    expect(screen.getByRole<HTMLSelectElement>('combobox').value).toBe('90');
  });

  it('can be disabled, and marked invalid with a message that describes it', () => {
    render(
      <>
        <SelectField
          label="Keep history for"
          options={OPTIONS}
          value="90"
          invalid
          describedBy="why"
          disabled
          onChange={vi.fn()}
        />
        <p id="why">Pick a length.</p>
      </>,
    );

    const select = screen.getByRole('combobox');
    expect(select.hasAttribute('disabled')).toBe(true);
    expect(select.getAttribute('aria-invalid')).toBe('true');
    expect(select.getAttribute('aria-describedby')).toBe('why');
  });

  it('has no accessibility violations', async () => {
    const { container } = render(
      <SelectField label="Keep history for" options={OPTIONS} value="90" onChange={vi.fn()} />,
    );

    await expectAccessible(container);
  });
});
