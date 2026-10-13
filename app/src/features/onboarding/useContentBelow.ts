import { type RefObject, useEffect, useState } from 'react';

const EDGE_TOLERANCE_PX = 1;

function hasContentBelow(element: HTMLElement): boolean {
  return element.scrollHeight - element.scrollTop - element.clientHeight > EDGE_TOLERANCE_PX;
}

/**
 * True while the scroller still has content under its bottom edge, which is when the footer needs its hairline.
 * Steps swap through child mutations and disclosures grow through the stage's size, and neither reads layout while a slide is animating styles.
 */
export function useContentBelow(ref: RefObject<HTMLElement | null>): boolean {
  const [below, setBelow] = useState(false);

  useEffect(() => {
    const element = ref.current;
    if (element === null) return;
    let frame = 0;
    const measure = (): void => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        setBelow(hasContentBelow(element));
      });
    };
    measure();
    const mutations = new MutationObserver(measure);
    mutations.observe(element, { subtree: true, childList: true });
    const sizes = typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(measure);
    if (element.firstElementChild !== null) sizes?.observe(element.firstElementChild);
    sizes?.observe(element);
    element.addEventListener('scroll', measure, { passive: true });
    return () => {
      cancelAnimationFrame(frame);
      mutations.disconnect();
      sizes?.disconnect();
      element.removeEventListener('scroll', measure);
    };
  }, [ref]);

  return below;
}
