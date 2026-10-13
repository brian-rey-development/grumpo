import type { BlockReason } from '@/ipc/generated/BlockReason';

export const HEADLINES = ['Nice try.', 'Nope.', 'Not today.'] as const;

export const blockerCopy = {
  body: {
    site: (domain: string) =>
      `You told Grumpo to block ${domain}. He's taking his job very seriously.`,
    app: (name: string) => `You told Grumpo to keep ${name} closed. So he closed it.`,
    limit: (target: string) => `That's all of today's ${target}. Grumpo is guarding the rest.`,
  },
  countdown: {
    schedule: 'Focus ends in',
    always: 'Focus ends in',
    limit: 'Resets in',
  } satisfies Record<BlockReason, string>,
  strict: 'Strict mode on',
  dismiss: 'Back to work',
  footer: "No, Grumpo doesn't take bribes.",
} as const;
