import { beforeEach, describe, expect, expectTypeOf, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn<(command: string, args?: Record<string, unknown>) => Promise<unknown>>(),
}));

vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));

import {
  type ArgsOf,
  type CommandMap,
  type CommandName,
  invokeCommand,
  type ResultOf,
} from '@/ipc/commands';
import { CommandFailure } from '@/ipc/errors';
import type { BlockScreen } from '@/ipc/generated/BlockScreen';
import type { DayReport } from '@/ipc/generated/DayReport';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';

beforeEach(() => {
  invokeMock.mockReset();
});

describe('CommandMap', () => {
  it('derives argument and result types from the contract', () => {
    expectTypeOf<ArgsOf<'get_day_report'>>().toEqualTypeOf<[args: { date: string }]>();
    expectTypeOf<ArgsOf<'get_limit_usage'>>().toEqualTypeOf<[]>();
    expectTypeOf<ResultOf<'get_day_report'>>().toEqualTypeOf<DayReport>();
    expectTypeOf<ResultOf<'relock'>>().toEqualTypeOf<FocusStatus>();
    expectTypeOf<ResultOf<'get_app_icon'>>().toEqualTypeOf<string | null>();
    expectTypeOf<ArgsOf<'install_service'>>().toEqualTypeOf<[]>();
    expectTypeOf<ResultOf<'install_service'>>().toEqualTypeOf<'installed' | 'cancelled'>();
    expectTypeOf<ResultOf<'get_block_screen'>>().toEqualTypeOf<BlockScreen | null>();
    expectTypeOf<ArgsOf<'dismiss_block_screen'>>().toEqualTypeOf<[args: { id?: number }]>();
    expectTypeOf<CommandName>().toEqualTypeOf<keyof CommandMap>();
  });

  it('rejects wrong arguments at compile time', () => {
    const never = async () => {
      // @ts-expect-error a command with arguments cannot be called without them
      await invokeCommand('get_day_report');
      // @ts-expect-error a command without arguments takes none
      await invokeCommand('relock', { date: 'x' });
      // @ts-expect-error the argument keys are camelCase
      await invokeCommand('get_week_report', { end_date: '2026-10-12' });
      // @ts-expect-error unknown commands do not exist
      await invokeCommand('get_everything');
    };
    expect(never).toBeTypeOf('function');
  });
});

describe('invokeCommand', () => {
  it('resolves with the command result and forwards arguments', async () => {
    invokeMock.mockResolvedValue({ totalSeconds: 42 });

    const result = await invokeCommand('get_day_report', { date: '2026-10-12' });

    expect(result).toEqual({ totalSeconds: 42 });
    expect(invokeMock).toHaveBeenCalledWith('get_day_report', { date: '2026-10-12' });
  });

  it('calls argument-less commands without a payload', async () => {
    invokeMock.mockResolvedValue([]);

    await invokeCommand('get_limit_usage');

    expect(invokeMock).toHaveBeenCalledWith('get_limit_usage', undefined);
  });

  it('turns a backend rejection into a CommandFailure that keeps the original as cause', async () => {
    const rejection = { code: 'looseningLocked', message: 'Rules are locked' };
    invokeMock.mockRejectedValue(rejection);

    const error: unknown = await invokeCommand('relock').catch((caught: unknown) => caught);

    expect(error).toBeInstanceOf(CommandFailure);
    expect(error).toMatchObject({ code: 'looseningLocked', message: 'Rules are locked' });
    expect(error).toHaveProperty('cause', rejection);
  });
});
