import { query, queryAll } from '@/scripts/lib/dom';

const COPIED = 'Copied';
const SELECTED_MESSAGE = 'Selected. Press Command C to copy.';
const RESET_AFTER_MS = 2_000;

function selectContents(element: HTMLElement): void {
  const selection = window.getSelection();
  const range = document.createRange();
  range.selectNodeContents(element);
  selection?.removeAllRanges();
  selection?.addRange(range);
}

function createFeedback(label: HTMLElement, status: HTMLElement) {
  const idleLabel = label.textContent;
  let reset: number | undefined;

  const show = (visibleLabel: string, message: string): void => {
    label.textContent = visibleLabel;
    status.textContent = message;
    window.clearTimeout(reset);
    reset = window.setTimeout(() => {
      label.textContent = idleLabel;
      status.textContent = '';
    }, RESET_AFTER_MS);
  };

  return {
    copied: () => show(COPIED, COPIED),
    selected: () => show(idleLabel, SELECTED_MESSAGE),
  };
}

function mountCopyCommand(root: HTMLElement): void {
  const source = query(root, '[data-copy-source]');
  const button = query(root, '[data-copy-button]', HTMLButtonElement);
  const feedback = createFeedback(
    query(root, '[data-copy-label]'),
    query(root, '[data-copy-status]'),
  );
  button.addEventListener('click', () => {
    navigator.clipboard.writeText(source.textContent).then(feedback.copied, () => {
      selectContents(source);
      feedback.selected();
    });
  });
  button.hidden = false;
}

export function mountCopyCommands(): void {
  if (!window.isSecureContext || !('clipboard' in navigator)) return;
  queryAll(document, '[data-copy]').forEach(mountCopyCommand);
}
