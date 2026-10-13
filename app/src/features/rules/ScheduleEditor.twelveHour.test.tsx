import { screen } from '@testing-library/react';
import { beforeEach, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, ...args: unknown[]) => Promise<unknown>>(),
}));

vi.mock('@/ipc/commands', () => ({ invokeCommand: invokeMock }));

import { ScheduleEditor } from '@/features/rules/ScheduleEditor';
import { EMPTY_RULES } from '@/test/ipc';
import { installDefaultLocale } from '@/test/locale';
import { installRulesCommands, renderEditor } from '@/test/rules';

installDefaultLocale('en-US');

beforeEach(() => {
  invokeMock.mockReset();
  installRulesCommands(invokeMock);
});

it('shows the example time in the 12 hour clock of the locale', () => {
  renderEditor((props) => <ScheduleEditor {...props} />, { initial: EMPTY_RULES });

  const text = screen.getByText(/Add a time like Monday to Friday/).textContent;
  expect(text.replace(/\s/g, ' ')).toContain('9:00 AM to 5:00 PM');
});
