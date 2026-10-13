import { Button } from '@/components/Button/Button';
import { EmptyState } from '@/components/EmptyState/EmptyState';
import { Mascot } from '@/components/Mascot/Mascot';
import { Section } from '@/components/Section/Section';
import { AppsEditor } from '@/features/rules/AppsEditor';
import { rulesCopy } from '@/features/rules/copy';
import { hasProblems } from '@/features/rules/problems';
import { LimitsEditor } from '@/features/rules/LimitsEditor';
import {
  DraftProblemsNotice,
  LooseningNotice,
  ReadOnlyNotice,
  SaveFailureNotice,
} from '@/features/rules/RulesNotices';
import { ScheduleEditor } from '@/features/rules/ScheduleEditor';
import { SitesEditor } from '@/features/rules/SitesEditor';
import { StrictPill } from '@/features/rules/StrictPill';
import { useRulesLocks } from '@/features/status/useRulesLocks';
import { useSaveFlow, type SaveFlow } from '@/features/rules/useSaveFlow';
import type { RulesDraft } from '@/features/rules/useRulesDraft';
import { useNavigation } from '@/layout/navigation';
import { ToolbarActions } from '@/layout/Toolbar';
import type { Rules } from '@/ipc/generated/Rules';

import styles from './Editors.module.css';

function SaveFailure({ flow }: { flow: SaveFlow }) {
  const { failure } = flow;
  if (failure === null) return null;
  if (failure.code !== 'looseningLocked') return <SaveFailureNotice failure={failure} />;
  if (!flow.locked) return null;
  return (
    <LooseningNotice
      onUnlock={flow.unlock}
      blocking={flow.blocking}
      includesLimitChange={flow.includesLimitChange}
    />
  );
}

interface SaveActionsProps {
  draft: RulesDraft;
  flow: SaveFlow;
}

function SaveActions({ draft, flow }: SaveActionsProps) {
  const copy = rulesCopy.screen;
  return (
    <ToolbarActions>
      <Button
        disabled={flow.saving}
        onClick={() => {
          flow.clearFailure();
          draft.discard();
        }}
      >
        {copy.discard}
      </Button>
      <Button
        variant="primary"
        loading={flow.saving}
        disabled={!flow.canSave}
        onClick={flow.submit}
      >
        {flow.needsUnlock ? copy.unlockToSave : copy.save}
      </Button>
    </ToolbarActions>
  );
}

function hasNoRules(rules: Rules): boolean {
  return (
    rules.blockedSites.length === 0 &&
    rules.blockedApps.length === 0 &&
    rules.schedule.windows.length === 0 &&
    rules.limits.length === 0
  );
}

interface RulesEditorProps {
  draft: RulesDraft;
}

/**
 * Shows the draft, or the saved rules alone while the service cannot be reached. Unsaved edits are
 * kept for when it comes back, but showing them next to a notice about saved rules would mislead.
 */
export function RulesEditor({ draft }: RulesEditorProps) {
  const flow = useSaveFlow(draft);
  const locks = useRulesLocks();
  const { navigate } = useNavigation();
  const { readOnly } = draft;
  const rules = readOnly ? draft.saved : draft.rules;
  if (rules === null) return null;
  const editorProps = {
    rules,
    readOnly,
    edit: (change: (current: Rules) => Rules) => {
      flow.clearFailure();
      draft.edit(change);
    },
  };

  return (
    <div className={styles.stack}>
      {locks.blocking || (locks.limits && rules.limits.length > 0) ? (
        <ToolbarActions>
          <StrictPill />
        </ToolbarActions>
      ) : null}
      {draft.dirty ? <SaveActions draft={draft} flow={flow} /> : null}
      {readOnly ? (
        <ReadOnlyNotice
          hasEdits={draft.dirty}
          onOpenSettings={() => {
            navigate('settings');
          }}
        />
      ) : null}
      <SaveFailure flow={flow} />
      {draft.dirty && !readOnly && hasProblems(rules) ? <DraftProblemsNotice /> : null}
      {hasNoRules(rules) ? (
        <Section>
          <EmptyState
            visual={<Mascot pose="stop" size="empty" />}
            title={rulesCopy.empty.title}
            description={rulesCopy.empty.description}
          />
        </Section>
      ) : null}
      <SitesEditor {...editorProps} />
      <AppsEditor {...editorProps} />
      <ScheduleEditor {...editorProps} />
      <LimitsEditor {...editorProps} />
    </div>
  );
}
