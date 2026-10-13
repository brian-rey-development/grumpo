import { commonCopy } from '@/copy/common';
import { CONFIRM_WINDOW_MINUTES, COOLDOWN_STEPS_MINUTES } from '@/ipc/generated/constants';
import { formatDurationLong } from '@/lib/format';
import { SECONDS_PER_MINUTE } from '@/lib/time';

function minutesText(minutes: number): string {
  return formatDurationLong(minutes * SECONDS_PER_MINUTE);
}

const MAX_WAIT_MINUTES = Math.max(...COOLDOWN_STEPS_MINUTES);
const confirmWindow = minutesText(CONFIRM_WINDOW_MINUTES);

export const unlockCopy = {
  titles: {
    locked: 'Unlock focus',
    coolingDown: 'Waiting to unlock',
    ready: 'Unlock ready',
    unlocked: 'Focus is unlocked',
    unavailable: commonCopy.serviceNotRunning,
    loading: 'Unlock focus',
    failed: 'Unlock focus',
  },
  reasonLabel: 'Change waiting to be saved',
  locked: {
    wait: (minutes: number) => `You'll wait ${minutesText(minutes)} before you can unlock.`,
    escalation: `Each unlock today makes the next wait longer, up to ${minutesText(MAX_WAIT_MINUTES)}.`,
    reset: 'The count starts over at midnight.',
    lapsed: `Your ${confirmWindow} to confirm ran out. Start again when you're ready.`,
    start: 'Start the wait',
    keep: 'Keep focusing',
  },
  coolingDown: {
    ringLabel: 'Time left before you can unlock',
    note: `You can close this. Grumpo will notify you when you can unlock, and you'll have ${confirmWindow} to confirm.`,
    cancel: 'Cancel unlock',
  },
  ready: {
    instruction: 'Type this sentence to continue',
    fieldLabel: 'Your sentence',
    matched: 'That matches.',
    durationLabel: 'Unlock for',
    confirm: (minutes: number) => `Unlock for ${minutesText(minutes)}`,
    deadline: (countdown: string) => `Confirm within ${countdown}`,
    timeUp: 'The time to confirm is up.',
    cancel: 'Cancel unlock',
  },
  unlocked: {
    remaining: (countdown: string) => `${countdown} left`,
    saveChanges: 'Save changes',
    relock: 'Relock now',
  },
  loading: 'Checking the Grumpo service',
  failed: { body: "Grumpo couldn't read its current state." },
  announcements: {
    locked: 'Focus is locked.',
    coolingDown: 'The wait has started.',
    ready: 'Unlock ready. Type the sentence to continue.',
    unlocked: 'Focus is unlocked.',
  },
} as const;
