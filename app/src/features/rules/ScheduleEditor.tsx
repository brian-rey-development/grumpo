import { Plus } from 'lucide-react';

import { Button } from '@/components/Button/Button';
import { EmptyState } from '@/components/EmptyState/EmptyState';
import { Section } from '@/components/Section/Section';
import { rulesCopy } from '@/features/rules/copy';
import type { EditorProps } from '@/features/rules/editorProps';
import { windowProblems } from '@/features/rules/problems';
import {
  addWindow,
  defaultWindow,
  removeWindow,
  RULE_CAPS,
  updateWindow,
} from '@/features/rules/rulesModel';
import { WeekPreview } from '@/features/rules/WeekPreview';
import { WindowRow } from '@/features/rules/WindowRow';
import { useClockOptions } from '@/ipc/queries';
import { formatTimeOfDay } from '@/lib/format';

import styles from './ScheduleEditor.module.css';

const ICON_SIZE = 14;

function EmptySchedule() {
  const clock = useClockOptions();
  const { start, end } = defaultWindow();
  const copy = rulesCopy.schedule;

  return (
    <EmptyState
      title={copy.emptyTitle}
      description={copy.emptyDescription(
        formatTimeOfDay(start, undefined, clock),
        formatTimeOfDay(end, undefined, clock),
      )}
    />
  );
}

export function ScheduleEditor({ rules, edit, readOnly }: EditorProps) {
  const copy = rulesCopy.schedule;
  const windows = rules.schedule.windows;
  const problems = windowProblems(rules);
  const atCap = windows.length >= RULE_CAPS.windows;
  const addButton = readOnly ? undefined : (
    <Button
      size="small"
      disabled={atCap}
      icon={<Plus size={ICON_SIZE} aria-hidden="true" />}
      onClick={() => {
        edit(addWindow);
      }}
    >
      {copy.add}
    </Button>
  );

  return (
    <Section
      title={copy.title}
      description={atCap ? copy.capReached(RULE_CAPS.windows) : copy.description}
      action={addButton}
    >
      {windows.length === 0 ? (
        <EmptySchedule />
      ) : (
        <div className={styles.content}>
          <WeekPreview schedule={rules.schedule} />
          <ul className={styles.rows}>
            {windows.map((window, index) => (
              <WindowRow
                // Windows have no identity of their own, so their position is the only key.
                key={index}
                number={index + 1}
                window={window}
                problem={problems[index] ?? null}
                readOnly={readOnly}
                onChange={(change) => {
                  edit((current) => updateWindow(current, index, change));
                }}
                onRemove={() => {
                  edit((current) => removeWindow(current, index));
                }}
              />
            ))}
          </ul>
        </div>
      )}
    </Section>
  );
}
