import { duplicateLimitCopy, siteProblemCopy } from '@/copy/rulesFaults';

export const rulesCopy = {
  screen: {
    save: 'Save changes',
    unlockToSave: 'Unlock to save',
    discard: 'Discard',
    loading: 'Loading your rules',
    loadFailedTitle: "Rules couldn't be loaded",
  },
  notices: {
    loosening: {
      title: 'Unlock to save these changes',
      body: "These changes weaken focus while it's on. Unlock to save them, or save them when the block ends.",
      limitsBody:
        'Raising or removing a daily limit always needs an unlock, even outside focus time. Unlock to save these changes.',
      fallbackSummary: 'These changes weaken your rules.',
      action: 'Start unlock',
    },
    saveFailedTitle: 'Rules not saved',
    draftProblems: 'Fix the focus times marked below to save your changes.',
    readOnly: {
      body: 'These are your rules as last saved. Start the Grumpo service to change them.',
      keptEdits: 'Your unsaved changes are kept and come back when the Grumpo service does.',
    },
  },
  strict: {
    label: 'Strict',
    hint: 'Removing or loosening this needs an unlock.',
  },
  empty: {
    title: 'No rules yet',
    description: 'Add a site, an app or a focus time and Grumpo starts saying no.',
  },
  unavailable: {
    title: "Rules aren't available yet",
    description: 'Grumpo can only read and change rules while its service is running.',
  },
  sites: {
    title: 'Blocked sites',
    description: 'Blocked during focus time, together with their www and mobile versions.',
    inputLabel: 'Site',
    placeholder: 'youtube.com',
    add: 'Add site',
    sheetTitle: 'Add a site',
    remove: (domain: string) => `Remove ${domain}`,
    emptyTitle: 'No blocked sites',
    emptyDescription: 'Add a site like youtube.com and Grumpo will keep it shut during focus time.',
    duplicate: (domain: string) => `${domain} is already blocked.`,
    capReached: (max: number) =>
      `You've reached the limit of ${max.toString()} sites. Remove one to add another.`,
  },
  domainErrors: siteProblemCopy,
  apps: {
    title: 'Blocked apps',
    description: 'Closed automatically during focus time.',
    add: 'Add app',
    remove: (name: string) => `Remove ${name}`,
    emptyTitle: 'No blocked apps',
    emptyDescription: 'Add an app and Grumpo will quit it whenever you open it during focus time.',
    capReached: (max: number) =>
      `You've reached the limit of ${max.toString()} apps. Remove one to add another.`,
    picker: {
      title: 'Add an app',
      searchLabel: 'Search apps',
      searchPlaceholder: 'Search by name',
      listLabel: 'Apps on this Mac',
      running: 'Running now',
      loadFailed: "Grumpo couldn't list the apps on this Mac.",
      noResults: (query: string) => `No app matches "${query}".`,
      noneLeft: 'Every app Grumpo found is already on the list.',
    },
  },
  schedule: {
    title: 'Focus schedule',
    description: 'Sites and apps on your lists are blocked during these times every week.',
    add: 'Add focus time',
    endOfDay: 'End of day',
    windowLabel: (number: number) => `Focus time ${number.toString()}`,
    daysLabel: (number: number) => `Days for focus time ${number.toString()}`,
    startLabel: (number: number) => `Start time for focus time ${number.toString()}`,
    endLabel: (number: number) => `End time for focus time ${number.toString()}`,
    to: 'to',
    remove: (number: number) => `Remove focus time ${number.toString()}`,
    emptyTitle: 'No focus times',
    emptyDescription: (start: string, end: string) =>
      `Add a time like Monday to Friday, ${start} to ${end}, and Grumpo starts saying no. Outside focus times only daily limits apply.`,
    capReached: (max: number) =>
      `You've reached the limit of ${max.toString()} focus times. Remove one to add another.`,
    problems: {
      noDays: 'Choose at least one day.',
      notForward: 'The end has to come after the start.',
    },
    preview: {
      none: 'no focus time',
      summary: (lines: string) => `Weekly focus times. ${lines}`,
      dayLine: (day: string, spans: string) => `${day} ${spans}.`,
    },
  },
  limits: {
    title: 'Daily limits',
    description: 'When the time is used up, the site or app stays blocked until midnight.',
    add: 'Add limit',
    remove: (name: string) => `Remove the limit for ${name}`,
    stepperLabel: (name: string) => `Daily limit for ${name}`,
    decrease: (name: string) => `Decrease the daily limit for ${name}`,
    increase: (name: string) => `Increase the daily limit for ${name}`,
    used: (duration: string) => `Used ${duration} today`,
    unused: 'Not used today',
    emptyTitle: 'No daily limits',
    emptyDescription:
      'Give a site or app a daily allowance, like 30 minutes of YouTube. Grumpo cuts it off after that.',
    capReached: (max: number) =>
      `You've reached the limit of ${max.toString()} daily limits. Remove one to add another.`,
    sheet: {
      title: 'Add a daily limit',
      description: 'Choose what to limit and how long it can be used each day.',
      kindLabel: 'Limit applies to',
      site: 'Site',
      app: 'App',
      siteLabel: 'Site',
      minutesLabel: 'Daily limit',
      decrease: 'Decrease the daily limit',
      increase: 'Increase the daily limit',
      noApp: 'Choose an app from the list.',
      change: 'Change',
      duplicate: duplicateLimitCopy,
      submit: 'Add limit',
    },
  },
  loosening: {
    removeSites: (names: string) => `Remove ${names}`,
    stopApps: (names: string) => `Stop blocking ${names}`,
    removeOneLimit: (name: string) => `Remove the limit for ${name}`,
    removeTwoLimits: (first: string, second: string) =>
      `Remove the limits for ${first} and ${second}`,
    removeManyLimits: (count: number) => `Remove ${count.toString()} daily limits`,
    raiseOneLimit: (name: string, from: string, to: string) =>
      `Raise the limit for ${name} from ${from} to ${to}`,
    raiseTwoLimits: (first: string, second: string) =>
      `Raise the limits for ${first} and ${second}`,
    raiseManyLimits: (count: number) => `Raise ${count.toString()} daily limits`,
    shortenSchedule: (duration: string) => `Cut ${duration} of focus time each week`,
    pair: (first: string, second: string) => `${first} and ${second}`,
    others: (first: string, count: number, noun: string) =>
      `${first} and ${count.toString()} other ${noun}s`,
  },
} as const;
