import type { Mock } from 'vitest';

import type { ArgsOf, CommandName, ResultOf } from '@/ipc/commands';

import { defaultHandlers } from './defaultHandlers';

export type CommandMock = Mock<(command: string, ...args: unknown[]) => Promise<unknown>>;

export type CommandHandlers = {
  [K in CommandName]?: (...args: ArgsOf<K>) => ResultOf<K> | Promise<ResultOf<K>>;
};

type AnyHandler = (...args: never[]) => unknown;

/**
 * Routes the mocked `invokeCommand` to per command handlers. Every read command has a quiet
 * default, so a test only states what it checks, and any command without a handler fails loudly.
 */
export function routeCommands(mock: CommandMock, overrides: CommandHandlers = {}): void {
  const table: Record<string, AnyHandler | undefined> = { ...defaultHandlers(), ...overrides };
  mock.mockImplementation(async (command, ...args) => {
    const handler = table[command];
    if (handler === undefined) throw new Error(`Unmocked command ${command}`);
    return await handler(...(args as never[]));
  });
}
