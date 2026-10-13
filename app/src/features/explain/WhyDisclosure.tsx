import { AnimatePresence, motion } from 'motion/react';
import { ChevronRight } from 'lucide-react';
import { useId, useState } from 'react';

import { tween, useMotionTransition } from '@/design/motion';

import { type Explanation, explainCopy } from './copy';

import styles from './WhyDisclosure.module.css';

const ICON_SIZE = 14;
const OPEN_TURN_DEGREES = 90;

type Details = Pick<Explanation, 'never' | 'without' | 'howTitle' | 'how'>;

function Reasons({ explanation }: { explanation: Details }) {
  const sections = [
    { title: explainCopy.neverTitle, body: explanation.never },
    { title: explainCopy.withoutTitle, body: explanation.without },
    { title: explanation.howTitle, body: explanation.how },
  ];

  return (
    <dl className={styles.list}>
      {sections.map((section) => (
        <div key={section.title} className={styles.item}>
          <dt className={styles.term}>{section.title}</dt>
          <dd className={styles.definition} data-selectable>
            {section.body}
          </dd>
        </div>
      ))}
    </dl>
  );
}

/** The reasons behind a permission or the service, tucked away so the row stays easy to scan. */
export function WhyDisclosure({ explanation }: { explanation: Details }) {
  const [open, setOpen] = useState(false);
  const panelId = useId();
  const slide = useMotionTransition(tween.base);
  const turn = useMotionTransition(tween.fast);

  return (
    <div className={styles.disclosure}>
      <button
        type="button"
        className={styles.summary}
        aria-expanded={open}
        aria-controls={open ? panelId : undefined}
        onClick={() => {
          setOpen((current) => !current);
        }}
      >
        <motion.span
          className={styles.chevron}
          initial={false}
          animate={{ rotate: open ? OPEN_TURN_DEGREES : 0 }}
          transition={turn}
        >
          <ChevronRight size={ICON_SIZE} aria-hidden="true" />
        </motion.span>
        {explainCopy.summary}
      </button>
      <AnimatePresence initial={false}>
        {open ? (
          <motion.div
            key="panel"
            id={panelId}
            className={styles.panel}
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={slide}
          >
            <Reasons explanation={explanation} />
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}
