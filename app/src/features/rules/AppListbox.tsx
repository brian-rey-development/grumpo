import { type KeyboardEvent, useEffect, useId, useRef, useState } from 'react';

import { TextField } from '@/components/TextField/TextField';
import { AppBadge } from '@/features/rules/AppBadge';
import { filterApps, optionId } from '@/features/rules/appList';
import { rulesCopy } from '@/features/rules/copy';
import type { AppRef } from '@/ipc/generated/AppRef';
import type { KnownApp } from '@/ipc/generated/KnownApp';

import styles from './AppListbox.module.css';

const copy = rulesCopy.apps.picker;

interface AppListboxProps {
  apps: readonly KnownApp[];
  excluded: ReadonlySet<string>;
  onPick: (app: AppRef) => void;
}

function nextIndex(key: string, current: number, count: number): number | null {
  switch (key) {
    case 'ArrowDown':
      return Math.min(current + 1, count - 1);
    case 'ArrowUp':
      return Math.max(current - 1, 0);
    case 'Home':
      return 0;
    case 'End':
      return count - 1;
    default:
      return null;
  }
}

function emptyMessage(query: string, anyApps: boolean): string {
  return query.trim().length > 0 && anyApps ? copy.noResults(query.trim()) : copy.noneLeft;
}

interface AppOptionProps {
  id: string;
  entry: KnownApp;
  selected: boolean;
  onHover: () => void;
  onPick: () => void;
}

function AppOption({ id, entry, selected, onHover, onPick }: AppOptionProps) {
  return (
    <button
      id={id}
      type="button"
      role="option"
      tabIndex={-1}
      className={styles.option}
      aria-selected={selected}
      onMouseMove={onHover}
      onClick={onPick}
    >
      <AppBadge app={entry.app} />
      <span className={styles.name}>{entry.app.name}</span>
      {entry.running ? <span className={styles.running}>{copy.running}</span> : null}
    </button>
  );
}

/** Keeps the highlighted option in view as the keyboard moves it. */
function useScrollIntoView(elementId: string | undefined): void {
  useEffect(() => {
    const element = elementId === undefined ? null : document.getElementById(elementId);
    if (typeof element?.scrollIntoView === 'function') element.scrollIntoView({ block: 'nearest' });
  }, [elementId]);
}

interface ListNavigation {
  activeIndex: number;
  activeId: string | undefined;
  setActive: (index: number) => void;
  reset: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLInputElement>) => void;
}

/** Moves a highlight through `options` with the arrow keys, and picks the highlighted app on Enter. */
function useListNavigation(
  options: readonly KnownApp[],
  listId: string,
  onPick: (app: AppRef) => void,
): ListNavigation {
  const [active, setActive] = useState(0);
  const activeIndex = Math.min(active, Math.max(options.length - 1, 0));
  const activeApp = options[activeIndex]?.app;
  const activeId = activeApp === undefined ? undefined : optionId(listId, activeApp);
  useScrollIntoView(activeId);

  function onKeyDown(event: KeyboardEvent<HTMLInputElement>): void {
    if (event.key === 'Enter') {
      event.preventDefault();
      if (activeApp !== undefined) onPick(activeApp);
      return;
    }
    const target = nextIndex(event.key, activeIndex, options.length);
    if (target === null || options.length === 0) return;
    event.preventDefault();
    setActive(target);
  }

  return {
    activeIndex,
    activeId,
    setActive,
    onKeyDown,
    reset: () => {
      setActive(0);
    },
  };
}

/** A search field that drives a list of apps from the keyboard, the way a combobox does. */
export function AppListbox({ apps, excluded, onPick }: AppListboxProps) {
  const listId = useId();
  const searchRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState('');
  const options = filterApps(apps, excluded, query);
  const nav = useListNavigation(options, listId, onPick);

  useEffect(() => {
    searchRef.current?.focus();
  }, []);

  return (
    <div className={styles.box}>
      <TextField
        ref={searchRef}
        label={copy.searchLabel}
        placeholder={copy.searchPlaceholder}
        value={query}
        role="combobox"
        aria-expanded="true"
        aria-controls={listId}
        aria-autocomplete="list"
        aria-activedescendant={nav.activeId}
        autoComplete="off"
        spellCheck={false}
        onChange={(event) => {
          setQuery(event.target.value);
          nav.reset();
        }}
        onKeyDown={nav.onKeyDown}
      />
      <div id={listId} className={styles.list} role="listbox" aria-label={copy.listLabel}>
        {options.map((entry, index) => (
          <AppOption
            key={entry.app.bundleId}
            id={optionId(listId, entry.app)}
            entry={entry}
            selected={index === nav.activeIndex}
            onHover={() => {
              nav.setActive(index);
            }}
            onPick={() => {
              onPick(entry.app);
            }}
          />
        ))}
      </div>
      {options.length === 0 ? (
        <p className={styles.empty}>{emptyMessage(query, apps.length > 0)}</p>
      ) : null}
    </div>
  );
}
