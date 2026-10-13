import { X } from 'lucide-react';
import { AnimatePresence, motion } from 'motion/react';
import { useEffect, useState } from 'react';
import { createPortal } from 'react-dom';

import { IconButton } from '@/components/IconButton/IconButton';
import type { ToastMessage } from '@/components/Toast/useToast';
import { spring, tween, useMotionTransition } from '@/design/motion';

import styles from './Toast.module.css';

const TOAST_DURATION_MS = 5_000;
const TOAST_OFFSET_PX = 12;
const ICON_SIZE = 14;

/** Whether the pointer or the keyboard is on the toast with this id, so a newer toast starts untouched. */
interface Touch {
  id: number;
  hovered: boolean;
  focused: boolean;
}

const UNTOUCHED: Touch = { id: -1, hovered: false, focused: false };

/** A message that is being read or pointed at must stay, so the countdown only runs while it is left alone. */
function useAutoDismiss(
  toast: ToastMessage | null,
  paused: boolean,
  durationMs: number,
  onDismiss: () => void,
): void {
  useEffect(() => {
    if (toast === null || paused) return undefined;
    const timer = window.setTimeout(onDismiss, durationMs);
    return () => {
      window.clearTimeout(timer);
    };
  }, [toast, paused, durationMs, onDismiss]);
}

interface ToastProps {
  toast: ToastMessage | null;
  dismissLabel: string;
  onDismiss: () => void;
  durationMs?: number;
}

/**
 * A brief, non-blocking message. The polite live region is always present, so a screen reader
 * announces the text when it appears without taking focus.
 */
export function Toast({
  toast,
  dismissLabel,
  onDismiss,
  durationMs = TOAST_DURATION_MS,
}: ToastProps) {
  const enter = useMotionTransition(spring.snappy);
  const exit = useMotionTransition(tween.fast);

  const [touch, setTouch] = useState<Touch>(UNTOUCHED);
  const paused = toast !== null && touch.id === toast.id && (touch.hovered || touch.focused);
  useAutoDismiss(toast, paused, durationMs, onDismiss);

  function touching(change: Partial<Pick<Touch, 'hovered' | 'focused'>>): void {
    if (toast === null) return;
    setTouch((current) => ({
      ...(current.id === toast.id ? current : UNTOUCHED),
      ...change,
      id: toast.id,
    }));
  }

  return createPortal(
    <div className={styles.region} aria-live="polite" aria-atomic="true">
      <AnimatePresence>
        {toast === null ? null : (
          <motion.div
            key={toast.id}
            className={styles.toast}
            initial={{ opacity: 0, y: TOAST_OFFSET_PX }}
            animate={{ opacity: 1, y: 0, transition: enter }}
            exit={{ opacity: 0, transition: exit }}
            onPointerEnter={() => {
              touching({ hovered: true });
            }}
            onPointerLeave={() => {
              touching({ hovered: false });
            }}
            onFocus={() => {
              touching({ focused: true });
            }}
            onBlur={() => {
              touching({ focused: false });
            }}
          >
            <span className={styles.text}>{toast.text}</span>
            <IconButton
              label={dismissLabel}
              size="small"
              icon={<X size={ICON_SIZE} aria-hidden="true" />}
              onClick={onDismiss}
            />
          </motion.div>
        )}
      </AnimatePresence>
    </div>,
    document.body,
  );
}
