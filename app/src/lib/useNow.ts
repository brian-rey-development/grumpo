import { useCallback, useMemo, useSyncExternalStore } from 'react';

import { isDocumentHidden } from '@/lib/documentHidden';
import { MS_PER_SECOND } from '@/lib/time';

interface Ticker {
  listeners: Set<() => void>;
  timer: number | null;
  onVisibilityChange: () => void;
}

const tickers = new Map<number, Ticker>();

function alignedNow(granularityMs: number): number {
  return Math.floor(Date.now() / granularityMs) * granularityMs;
}

function notify(ticker: Ticker): void {
  for (const listener of ticker.listeners) listener();
}

function schedule(granularityMs: number, ticker: Ticker): void {
  if (ticker.timer !== null || isDocumentHidden()) return;
  const untilNextBoundary = granularityMs - (Date.now() % granularityMs);
  ticker.timer = window.setTimeout(() => {
    ticker.timer = null;
    notify(ticker);
    schedule(granularityMs, ticker);
  }, untilNextBoundary);
}

function pause(ticker: Ticker): void {
  if (ticker.timer === null) return;
  window.clearTimeout(ticker.timer);
  ticker.timer = null;
}

function createTicker(granularityMs: number): Ticker {
  const ticker: Ticker = { listeners: new Set(), timer: null, onVisibilityChange: () => undefined };
  ticker.onVisibilityChange = () => {
    if (isDocumentHidden()) {
      pause(ticker);
      return;
    }
    notify(ticker);
    schedule(granularityMs, ticker);
  };
  return ticker;
}

function subscribeTo(granularityMs: number, listener: () => void): () => void {
  const ticker = tickers.get(granularityMs) ?? createTicker(granularityMs);
  tickers.set(granularityMs, ticker);
  if (ticker.listeners.size === 0) {
    document.addEventListener('visibilitychange', ticker.onVisibilityChange);
  }
  ticker.listeners.add(listener);
  schedule(granularityMs, ticker);

  return () => {
    ticker.listeners.delete(listener);
    if (ticker.listeners.size > 0) return;
    document.removeEventListener('visibilitychange', ticker.onVisibilityChange);
    pause(ticker);
    tickers.delete(granularityMs);
  };
}

/**
 * The current time, floored to `granularityMs`. Every component asking for the same granularity
 * shares one timer aligned to the clock boundary, and the timer sleeps while the page is hidden.
 */
export function useNow(granularityMs: number = MS_PER_SECOND): Date {
  const subscribe = useCallback(
    (listener: () => void) => subscribeTo(granularityMs, listener),
    [granularityMs],
  );
  const timestamp = useSyncExternalStore(subscribe, () => alignedNow(granularityMs));
  return useMemo(() => new Date(timestamp), [timestamp]);
}
