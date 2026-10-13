export const blockDemo = {
  blocked: ['reddit.com', 'youtube.com', 'x.com', 'instagram.com', 'tiktok.com'],
  sites: ['reddit.com', 'youtube.com', 'github.com'],
  headlines: ['Nice try.', 'Nope.', 'Not today.'],
  allowedHeadline: 'Fine. Go ahead.',
  focusSecondsLeft: 5_398,
} as const;

export const unlockDemo = {
  sentence: 'I am choosing to give up my focus time, and I accept that cost.',
  cooldowns: [
    { label: '1st unlock today', minutes: 5 },
    { label: '2nd', minutes: 10 },
    { label: '3rd', minutes: 20 },
    { label: '4th and up', minutes: 40 },
  ],
  unlockMinutes: [15, 30, 60],
  speedup: 60,
} as const;
