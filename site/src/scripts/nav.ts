const SCROLLED_AT_PX = 8;

export function mountNav(): void {
  const header = document.querySelector<HTMLElement>('[data-nav]');
  if (header === null) return;
  const sync = () => header.toggleAttribute('data-scrolled', window.scrollY > SCROLLED_AT_PX);
  window.addEventListener('scroll', sync, { passive: true });
  sync();
}
