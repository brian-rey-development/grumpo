import { query, queryAll } from '@/scripts/lib/dom';

function closeOnBackdrop(dialog: HTMLDialogElement): void {
  let pressedBackdrop = false;
  dialog.addEventListener('pointerdown', (event) => {
    pressedBackdrop = event.target === dialog;
  });
  dialog.addEventListener('click', (event) => {
    if (pressedBackdrop && event.target === dialog) dialog.close();
  });
}

export function mountInstallHelp(): void {
  const dialog = query(document, '[data-install-help]', HTMLDialogElement);
  let opener: HTMLElement | undefined;
  queryAll(document, '[data-download]').forEach((link) => {
    link.addEventListener('click', () => {
      opener = link;
      if (!dialog.open) dialog.showModal();
    });
  });
  dialog.addEventListener('close', () => opener?.focus());
  closeOnBackdrop(dialog);
}
