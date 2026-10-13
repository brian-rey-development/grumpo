type ShortcutEvent = Pick<KeyboardEvent, 'key' | 'metaKey' | 'ctrlKey' | 'altKey' | 'shiftKey'>;

/** Cmd+S on its own, so Cmd+Shift+S and Cmd+Option+S stay free for whatever else wants them. */
export function isSaveShortcut(event: ShortcutEvent): boolean {
  if (!event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return false;
  return event.key.toLowerCase() === 's';
}
