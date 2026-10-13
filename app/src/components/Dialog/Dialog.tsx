import { AnimatePresence, motion, useIsPresent } from 'motion/react';
import { type ReactNode, type RefObject, useId, useRef } from 'react';
import { createPortal } from 'react-dom';

import { useModal } from '@/components/Dialog/useModal';
import { spring, tween, useMotionTransition } from '@/design/motion';

import styles from './Dialog.module.css';

const SHEET_HIDDEN_OFFSET = '-100%';
const SHEET_EXIT_OFFSET = -24;

interface DialogProps {
  open: boolean;
  onClose: () => void;
  title: string;
  description?: string;
  dismissible?: boolean;
  footer?: ReactNode;
  /** A mascot or picture shown above the title, which then takes the display type and centers. */
  hero?: ReactNode;
  initialFocusRef?: RefObject<HTMLElement | null>;
  children?: ReactNode;
}

type PanelProps = Pick<DialogProps, 'title' | 'description' | 'footer' | 'hero' | 'children'> & {
  panelRef: RefObject<HTMLDivElement | null>;
};

function DialogPanel({ panelRef, title, description, footer, hero, children }: PanelProps) {
  const titleId = useId();
  const descriptionId = useId();
  const enter = useMotionTransition(spring.snappy);
  const exit = useMotionTransition(tween.base);

  return (
    <motion.div
      ref={panelRef}
      className={styles.panel}
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      aria-describedby={description === undefined ? undefined : descriptionId}
      tabIndex={-1}
      data-hero={hero === undefined ? undefined : ''}
      initial={{ y: SHEET_HIDDEN_OFFSET }}
      animate={{ y: 0, transition: enter }}
      exit={{ y: SHEET_EXIT_OFFSET, opacity: 0, transition: exit }}
    >
      {hero === undefined ? null : <div className={styles.hero}>{hero}</div>}
      <h2 id={titleId} className={styles.title}>
        {title}
      </h2>
      {description === undefined ? null : (
        <p id={descriptionId} className={styles.description} data-selectable>
          {description}
        </p>
      )}
      {children === undefined ? null : <div className={styles.body}>{children}</div>}
      {footer === undefined ? null : <footer className={styles.footer}>{footer}</footer>}
    </motion.div>
  );
}

function DialogLayer({
  onClose,
  dismissible = true,
  initialFocusRef,
  ...panelContent
}: Omit<DialogProps, 'open'>) {
  const panelRef = useRef<HTMLDivElement>(null);
  const enter = useMotionTransition(tween.base);
  const exit = useMotionTransition(tween.base);
  const present = useIsPresent();
  useModal(panelRef, present, initialFocusRef, dismissible ? onClose : undefined);

  return (
    <motion.div
      className={styles.layer}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1, transition: enter }}
      exit={{ opacity: 0, transition: exit }}
    >
      <div
        className={styles.backdrop}
        role="presentation"
        data-testid="dialog-backdrop"
        onClick={dismissible ? onClose : undefined}
      />
      <div className={styles.stage}>
        <DialogPanel panelRef={panelRef} {...panelContent} />
      </div>
    </motion.div>
  );
}

export function Dialog({ open, ...layerProps }: DialogProps) {
  return createPortal(
    <AnimatePresence>{open ? <DialogLayer key="dialog" {...layerProps} /> : null}</AnimatePresence>,
    document.body,
  );
}
