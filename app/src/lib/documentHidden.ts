/** True when the window cannot be seen, such as at login or in a background tab, so animations would never run. */
export function isDocumentHidden(): boolean {
  return document.hidden;
}
