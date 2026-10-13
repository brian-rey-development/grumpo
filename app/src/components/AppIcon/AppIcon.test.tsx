import { fireEvent, render } from '@testing-library/react';
import { expect, it } from 'vitest';

import { AppIcon } from '@/components/AppIcon/AppIcon';
import { expectAccessible } from '@/test/render';

const DATA_URL = 'data:image/png;base64,iVBORw0KGgo=';

function image(container: HTMLElement): HTMLImageElement | null {
  return container.querySelector('img');
}

it('shows the icon as a decorative image once it has loaded', () => {
  const { container } = render(<AppIcon src={DATA_URL} />);
  const picture = image(container);

  expect(picture?.getAttribute('src')).toBe(DATA_URL);
  expect(picture?.getAttribute('alt')).toBe('');
  expect(picture?.hasAttribute('data-ready')).toBe(false);

  if (picture !== null) fireEvent.load(picture);

  expect(image(container)?.hasAttribute('data-ready')).toBe(true);
});

it('keeps a neutral glyph, and no letter, when there is no icon', () => {
  const { container } = render(<AppIcon src={null} />);

  expect(image(container)).toBeNull();
  expect(container.querySelector('svg')).not.toBeNull();
  expect(container.textContent).toBe('');
});

it('falls back to the glyph when the image fails to load', () => {
  const { container } = render(<AppIcon src={DATA_URL} />);
  const picture = image(container);

  if (picture !== null) fireEvent.error(picture);

  expect(image(container)).toBeNull();
  expect(container.querySelector('svg')).not.toBeNull();
});

it('has no accessibility violations in either form', async () => {
  const { container } = render(
    <>
      <AppIcon src={DATA_URL} />
      <AppIcon src={null} />
    </>,
  );

  await expectAccessible(container);
});
