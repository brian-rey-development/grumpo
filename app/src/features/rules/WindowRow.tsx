import { X } from 'lucide-react';
import { useId } from 'react';

import { DayPicker } from '@/components/DayPicker/DayPicker';
import { IconButton } from '@/components/IconButton/IconButton';
import { TimeField } from '@/components/TimeField/TimeField';
import { rulesCopy } from '@/features/rules/copy';
import type { WindowProblem } from '@/features/rules/problems';
import type { Window } from '@/ipc/generated/Window';
import { useClockStyle } from '@/ipc/queries';
import { TIME_STEP_MINUTES } from '@/lib/time';

import styles from './ScheduleEditor.module.css';

const ICON_SIZE = 14;
const copy = rulesCopy.schedule;

interface WindowTimesProps {
  number: number;
  window: Window;
  invalid: boolean;
  describedBy: string | undefined;
  disabled: boolean;
  onChange: (change: Partial<Window>) => void;
}

function WindowTimes({
  number,
  window,
  invalid,
  describedBy,
  disabled,
  onChange,
}: WindowTimesProps) {
  const clockStyle = useClockStyle().data;

  return (
    <div className={styles.times}>
      <TimeField
        label={copy.startLabel(number)}
        value={window.start}
        clockStyle={clockStyle}
        invalid={invalid}
        describedBy={describedBy}
        disabled={disabled}
        onChange={(start) => {
          onChange({ start });
        }}
      />
      <span className={styles.to}>{copy.to}</span>
      <TimeField
        label={copy.endLabel(number)}
        value={window.end}
        min={TIME_STEP_MINUTES}
        endOfDayLabel={copy.endOfDay}
        clockStyle={clockStyle}
        invalid={invalid}
        describedBy={describedBy}
        disabled={disabled}
        onChange={(end) => {
          onChange({ end });
        }}
      />
    </div>
  );
}

interface WindowRowProps {
  /** One-based position, used in names so screen reader users can tell the rows apart. */
  number: number;
  window: Window;
  problem: WindowProblem | null;
  readOnly: boolean;
  onChange: (change: Partial<Window>) => void;
  onRemove: () => void;
}

export function WindowRow({
  number,
  window,
  problem,
  readOnly,
  onChange,
  onRemove,
}: WindowRowProps) {
  const problemId = useId();
  const describedBy = (affected: WindowProblem) => (problem === affected ? problemId : undefined);

  return (
    <li className={styles.row}>
      <div className={styles.fields} role="group" aria-label={copy.windowLabel(number)}>
        <DayPicker
          label={copy.daysLabel(number)}
          value={window.days}
          invalid={problem === 'noDays'}
          describedBy={describedBy('noDays')}
          disabled={readOnly}
          onChange={(days) => {
            onChange({ days });
          }}
        />
        <WindowTimes
          number={number}
          window={window}
          invalid={problem === 'notForward'}
          describedBy={describedBy('notForward')}
          disabled={readOnly}
          onChange={onChange}
        />
        {readOnly ? null : (
          <IconButton
            label={copy.remove(number)}
            icon={<X size={ICON_SIZE} aria-hidden="true" />}
            onClick={onRemove}
          />
        )}
      </div>
      {problem === null ? null : (
        <p id={problemId} className={styles.problem} role="alert">
          {copy.problems[problem]}
        </p>
      )}
    </li>
  );
}
