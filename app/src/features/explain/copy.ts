export interface Explanation {
  purpose: string;
  never: string;
  without: string;
  howTitle: string;
  how: string;
}

export const explainCopy = {
  summary: 'Why Grumpo needs this',
  neverTitle: 'What Grumpo never does',
  withoutTitle: 'Without it',
  pathLabel: 'In System Settings',
  allowed: 'Allowed',
  service: {
    purpose:
      "Keeps your rules and the blocked-site list, and enforces them even when Grumpo is quit. Without an unlock, it won't loosen them.",
    never:
      'Grumpo never sees your password, because the prompt belongs to macOS. The service only holds your rules and the blocked-site list, and nothing leaves this Mac.',
    without:
      'Nothing gets blocked, because only the service is allowed to change the blocked-site list.',
    howTitle: 'How to install it',
    how: 'Choose Install service. macOS asks for your Mac password once.',
  },
  accessibility: {
    purpose:
      "Lets Grumpo see which app is in front, so it can count your time and quit apps you've blocked. It also reads the address bar in Firefox.",
    never:
      "Grumpo never reads what's inside a window, records keystrokes or takes screenshots. Nothing leaves this Mac.",
    without:
      "Time in apps isn't tracked and blocked apps aren't quit. Site blocking by the service still applies.",
    howTitle: 'How to allow it',
    how: 'Choose Allow. macOS opens System Settings, where you turn on Grumpo under Privacy and Security, then Accessibility.',
  },
  automation: (browser: string): Explanation => ({
    purpose: `Lets Grumpo read the site of the tab you're on in ${browser} and close a tab on a blocked site, in the frontmost window.`,
    never:
      'Grumpo only reads the site of the current tab. It never reads page content, records keystrokes or takes screenshots, and nothing leaves this Mac.',
    without: `Blocked tabs stay open in ${browser}, and time there isn't tracked. Site blocking by the service still applies.`,
    howTitle: 'How to allow it',
    how: `Choose Allow, and macOS asks whether Grumpo can control ${browser}. If you said no before, turn Grumpo on under Privacy and Security, then Automation, in System Settings.`,
  }),
  accessibilityPath: ['Privacy and Security', 'Accessibility', 'Grumpo'],
} as const;
