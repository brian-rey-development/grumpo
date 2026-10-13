export const onboardingCopy = {
  progressLabel: 'Progress',
  stepOf: (step: number, total: number) => `Step ${step.toString()} of ${total.toString()}`,
  back: 'Back',
  getStarted: 'Get started',
  next: 'Continue',
  later: 'Do this later',
  addRule: 'Add your first rule',
  goToday: 'Go to Today',
  saveFailed: "Grumpo couldn't save that.",
  loading: 'Loading',
  welcome: {
    title: 'Grumpo',
    tagline: ['Grumpo says no.', 'You get things done.'],
    descriptor: 'A strict focus blocker for your Mac.',
  },
  how: {
    title: 'How Grumpo works',
    points: [
      {
        object: 'block',
        title: 'Block',
        body: 'The sites and apps you pick stay closed during your hours.',
      },
      {
        object: 'limit',
        title: 'Limit',
        body: "A daily time per site or app, then it's locked until midnight.",
      },
      {
        object: 'unlock',
        title: 'Unlock',
        body: 'Loosening a rule takes a cooldown and a typed sentence.',
      },
    ],
    ruleLead: 'Tightening is instant.',
    rule: 'Loosening needs an unlock.',
  },
  private: {
    title: 'Private by design',
    lede: 'Everything stays on this Mac. Grumpo keeps which app and site you were in, per minute, for 90 days by default.',
    recordsTitle: 'What Grumpo records',
    records: ['Apps', 'Sites'],
    neverTitle: 'What Grumpo never touches',
    never: ['Page content', 'Keystrokes', 'Screenshots'],
  },
  service: {
    title: 'Install the Grumpo service',
    lede: "Grumpo can't block sites on his own. A small background service with admin rights does it, so macOS asks for your Mac password once.",
    label: 'Grumpo service',
    running: 'Running',
    runCommand:
      'Run this command in Terminal from the Grumpo project folder. This step updates once the service is running.',
  },
  permissions: {
    title: 'Allow what Grumpo needs',
    lede: 'All optional. Open the reasons under a row to see how Grumpo uses it.',
  },
  done: {
    title: "Grumpo's on duty.",
    lede: "Add your first rule and he'll start saying no.",
    recapTitle: 'What is set up',
    laterNote: 'You can do this in Settings.',
    service: { done: 'Running', pending: 'Not installed' },
    accessibility: { done: 'Allowed', pending: 'Not allowed yet' },
    browser: { done: 'Allowed', pending: 'Not allowed yet' },
  },
} as const;
