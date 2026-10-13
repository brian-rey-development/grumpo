import { motion } from 'motion/react';
import { type KeyboardEvent, type ReactNode, useId, useRef } from 'react';

import { spring, useMotionTransition } from '@/design/motion';

import styles from './SegmentedControl.module.css';

export interface SegmentedOption<TValue extends string> {
  value: TValue;
  label: ReactNode;
  ariaLabel?: string;
}

interface SegmentedControlProps<TValue extends string> {
  label: string;
  options: readonly SegmentedOption<TValue>[];
  value: TValue;
  onChange: (value: TValue) => void;
}

function targetIndex(key: string, current: number, count: number): number | null {
  switch (key) {
    case 'ArrowRight':
    case 'ArrowDown':
      return (current + 1) % count;
    case 'ArrowLeft':
    case 'ArrowUp':
      return (current - 1 + count) % count;
    case 'Home':
      return 0;
    case 'End':
      return count - 1;
    default:
      return null;
  }
}

interface SegmentButtonProps {
  option: SegmentedOption<string>;
  selected: boolean;
  indicatorId: string;
  buttonRef: (element: HTMLButtonElement | null) => void;
  onSelect: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLButtonElement>) => void;
}

function SegmentButton({
  option,
  selected,
  indicatorId,
  buttonRef,
  onSelect,
  onKeyDown,
}: SegmentButtonProps) {
  const transition = useMotionTransition(spring.snappy);

  return (
    <button
      ref={buttonRef}
      type="button"
      role="radio"
      className={styles.option}
      aria-checked={selected}
      aria-label={option.ariaLabel}
      tabIndex={selected ? 0 : -1}
      onClick={onSelect}
      onKeyDown={onKeyDown}
    >
      {selected ? (
        <motion.span layoutId={indicatorId} className={styles.indicator} transition={transition} />
      ) : null}
      <span className={styles.content}>{option.label}</span>
    </button>
  );
}

export function SegmentedControl<TValue extends string>({
  label,
  options,
  value,
  onChange,
}: SegmentedControlProps<TValue>) {
  const indicatorId = useId();
  const buttons = useRef<(HTMLButtonElement | null)[]>([]);
  const selectedIndex = Math.max(
    options.findIndex((option) => option.value === value),
    0,
  );

  function onKeyDown(event: KeyboardEvent<HTMLButtonElement>, index: number): void {
    const next = targetIndex(event.key, index, options.length);
    const option = next === null ? undefined : options[next];
    if (next === null || option === undefined) return;
    event.preventDefault();
    onChange(option.value);
    buttons.current[next]?.focus();
  }

  return (
    <div className={styles.control} role="radiogroup" aria-label={label}>
      {options.map((option, index) => (
        <SegmentButton
          key={option.value}
          option={option}
          selected={index === selectedIndex}
          indicatorId={indicatorId}
          buttonRef={(element) => {
            buttons.current[index] = element;
          }}
          onSelect={() => {
            onChange(option.value);
          }}
          onKeyDown={(event) => {
            onKeyDown(event, index);
          }}
        />
      ))}
    </div>
  );
}
