import type { CommandError } from '@/ipc/generated/CommandError';
import type { CommandErrorCode } from '@/ipc/generated/CommandErrorCode';
import type { RulesFault } from '@/ipc/generated/RulesFault';

/** Fails to compile when the generated `CommandErrorCode` gains or loses a variant. */
const KNOWN_CODES = {
  daemonUnavailable: true,
  versionMismatch: true,
  rulesInvalid: true,
  looseningLocked: true,
  unlockNotReady: true,
  unlockNotRequested: true,
  unlockAlreadyPending: true,
  unlockAlreadyActive: true,
  phraseMismatch: true,
  durationInvalid: true,
  invalidInput: true,
  storage: true,
  exportDenied: true,
  serviceInstallUnavailable: true,
  serviceInstallFailed: true,
  serviceInstallBusy: true,
  platform: true,
  internal: true,
} as const satisfies Record<CommandErrorCode, true>;

export const UNKNOWN_ERROR_CODE = 'unknown';

type FailureCode = CommandErrorCode | typeof UNKNOWN_ERROR_CODE;

const UNKNOWN_MESSAGE = 'Something went wrong';

/** Carries the service's own `message` for logs. Screens show wording chosen from `code` instead. */
export class CommandFailure extends Error {
  readonly code: FailureCode;
  readonly rulesFault: RulesFault | null;

  constructor(
    code: FailureCode,
    message: string,
    options?: ErrorOptions & { rulesFault?: RulesFault | null },
  ) {
    super(message, options);
    this.name = 'CommandFailure';
    this.code = code;
    this.rulesFault = options?.rulesFault ?? null;
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function isCommandError(value: unknown): value is CommandError {
  return (
    isRecord(value) &&
    typeof value.code === 'string' &&
    Object.hasOwn(KNOWN_CODES, value.code) &&
    typeof value.message === 'string' &&
    (value.rulesFault === undefined || value.rulesFault === null || isRulesFault(value.rulesFault))
  );
}

function isRulesFault(value: unknown): value is RulesFault {
  return isRecord(value) && typeof value.kind === 'string';
}

function messageOf(rejection: unknown): string | null {
  if (typeof rejection === 'string') return rejection.length > 0 ? rejection : null;
  if (rejection instanceof Error) return rejection.message.length > 0 ? rejection.message : null;
  if (
    isRecord(rejection) &&
    typeof rejection.message === 'string' &&
    rejection.message.length > 0
  ) {
    return rejection.message;
  }
  return null;
}

export function toCommandFailure(rejection: unknown): CommandFailure {
  if (rejection instanceof CommandFailure) return rejection;
  const options = { cause: rejection };
  if (isCommandError(rejection)) {
    return new CommandFailure(rejection.code, rejection.message, {
      ...options,
      rulesFault: rejection.rulesFault,
    });
  }
  return new CommandFailure(UNKNOWN_ERROR_CODE, messageOf(rejection) ?? UNKNOWN_MESSAGE, options);
}
