import { type ReactNode, useEffect } from 'react';

import type { EditorProps } from '@/features/rules/editorProps';
import { useRulesDraft } from '@/features/rules/useRulesDraft';
import type { Rules } from '@/ipc/generated/Rules';
import { type CommandHandlers, type CommandMock, routeCommands, YOUTUBE_USAGE } from '@/test/ipc';
import { SAMPLE_RULES } from '@/test/ipc';
import { type ProviderRenderResult, renderWithProviders } from '@/test/render';

/** Answers the queries every rules editor makes, so a test only has to add what it checks. */
export function installRulesCommands(mock: CommandMock, handlers: CommandHandlers = {}): void {
  routeCommands(mock, { get_limit_usage: () => YOUTUBE_USAGE, ...handlers });
}

interface HarnessProps {
  initial: Rules;
  readOnly: boolean;
  onRules: (rules: Rules) => void;
  children: (props: EditorProps) => ReactNode;
}

function EditorHarness({ initial, readOnly, onRules, children }: HarnessProps) {
  const draft = useRulesDraft(initial);
  const { rules } = draft;
  useEffect(() => {
    if (rules !== null) onRules(rules);
  }, [rules, onRules]);
  if (rules === null) throw new Error('The harness was given no rules to edit');
  return children({ rules, edit: draft.edit, readOnly });
}

interface EditorOptions {
  initial?: Rules;
  readOnly?: boolean;
}

export type RenderedEditor = ProviderRenderResult & {
  /** The rules as the editor has changed them so far. */
  currentRules: () => Rules;
};

/** Hosts an editor on a real draft, so a test can change it and then read back the resulting rules. */
export function renderEditor(
  editor: (props: EditorProps) => ReactNode,
  { initial = SAMPLE_RULES, readOnly = false }: EditorOptions = {},
): RenderedEditor {
  let latest: Rules | null = null;
  const rendered = renderWithProviders(
    <EditorHarness
      initial={initial}
      readOnly={readOnly}
      onRules={(rules) => {
        latest = rules;
      }}
    >
      {editor}
    </EditorHarness>,
  );
  return {
    ...rendered,
    currentRules: () => {
      if (latest === null) throw new Error('The editor has not rendered any rules yet');
      return latest;
    },
  };
}
