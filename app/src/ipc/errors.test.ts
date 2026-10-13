import { describe, expect, it } from 'vitest';

import { CommandFailure, toCommandFailure } from '@/ipc/errors';

describe('toCommandFailure', () => {
  it('keeps the code and message of a backend error and the original rejection as cause', () => {
    const rejection = { code: 'looseningLocked', message: 'Rules are locked' };

    const failure = toCommandFailure(rejection);

    expect(failure).toBeInstanceOf(CommandFailure);
    expect(failure.code).toBe('looseningLocked');
    expect(failure.message).toBe('Rules are locked');
    expect(failure.cause).toBe(rejection);
  });

  it.each([
    ['a code in the wrong case', { code: 'loosening_locked', message: 'b' }],
    ['a code that is not a string', { code: 7, message: 'b' }],
    ['no message', { code: 'internal' }],
    ['a code that only exists on Object', { code: 'toString', message: 'b' }],
  ])('treats %s as unknown', (_, rejection) => {
    expect(toCommandFailure(rejection).code).toBe('unknown');
  });

  it('accepts every code the backend can send', () => {
    for (const code of ['internal', 'rulesInvalid', 'daemonUnavailable']) {
      expect(toCommandFailure({ code, message: 'm' }).code).toBe(code);
    }
  });

  it('keeps the specific fault of refused rules, and none when there is none', () => {
    const fault = { kind: 'windowNotForward', number: 2 } as const;

    expect(
      toCommandFailure({ code: 'rulesInvalid', message: 'm', rulesFault: fault }).rulesFault,
    ).toEqual(fault);
    expect(
      toCommandFailure({ code: 'rulesInvalid', message: 'm', rulesFault: null }).rulesFault,
    ).toBeNull();
    expect(toCommandFailure({ code: 'rulesInvalid', message: 'm' }).rulesFault).toBeNull();
    expect(toCommandFailure({ code: 'rulesInvalid', message: 'm', rulesFault: 7 }).code).toBe(
      'unknown',
    );
  });

  it('returns an existing failure untouched', () => {
    const failure = new CommandFailure('storage', 'disk full');
    expect(toCommandFailure(failure)).toBe(failure);
  });

  it('keeps the message of an unknown code but flags the code as unknown', () => {
    const failure = toCommandFailure({ code: 'brandNew', message: 'From a newer backend' });

    expect(failure.code).toBe('unknown');
    expect(failure.message).toBe('From a newer backend');
  });

  it('wraps string and Error rejections', () => {
    expect(toCommandFailure('command not found').message).toBe('command not found');
    const error = new Error('socket closed');
    const failure = toCommandFailure(error);
    expect(failure.message).toBe('socket closed');
    expect(failure.cause).toBe(error);
    expect(failure.code).toBe('unknown');
  });

  it.each([[null], [undefined], [42], [{ code: 7, message: '' }], ['']])(
    'falls back to a generic error for the unrecognised rejection %j',
    (rejection) => {
      const failure = toCommandFailure(rejection);

      expect(failure.code).toBe('unknown');
      expect(failure.message).toBe('Something went wrong');
      expect(failure.cause).toBe(rejection);
    },
  );
});
