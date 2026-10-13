type ElementType<T extends Element> = abstract new (...args: never[]) => T;

export function query(root: ParentNode, selector: string): HTMLElement;
export function query<T extends Element>(
  root: ParentNode,
  selector: string,
  type: ElementType<T>,
): T;
export function query(
  root: ParentNode,
  selector: string,
  type: ElementType<Element> = HTMLElement,
): Element {
  const element = root.querySelector(selector);
  if (!(element instanceof type)) throw new Error(`Missing ${type.name} ${selector}`);
  return element;
}

export function queryAll(root: ParentNode, selector: string): HTMLElement[];
export function queryAll<T extends Element>(
  root: ParentNode,
  selector: string,
  type: ElementType<T>,
): T[];
export function queryAll(
  root: ParentNode,
  selector: string,
  type: ElementType<Element> = HTMLElement,
): Element[] {
  return Array.from(root.querySelectorAll(selector)).filter((element) => element instanceof type);
}
