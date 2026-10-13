import { site } from '@/config/site';

export interface TourStep {
  id: string;
  title: string;
  body: string;
  screenshot: { src: string; alt: string };
  mascot: { src: string; width: number; height: number };
}

export const tourSteps: readonly TourStep[] = [
  {
    id: 'see',
    title: 'Track your time',
    body: 'See every app and site you used today, by the hour.',
    screenshot: {
      src: 'img/today.webp',
      alt: 'The Today screen with 1 hour 40 minutes of screen time, an hourly chart and a banner saying Grumpo is on duty until 17:00',
    },
    mascot: { src: 'img/sticker-laptop.webp', width: 377, height: 275 },
  },
  {
    id: 'set',
    title: 'Set rules',
    body: 'Block sites and apps on a schedule, or give them a daily limit.',
    screenshot: {
      src: 'img/rules.webp',
      alt: 'The Rules screen listing reddit.com, youtube.com and Slack as blocked, with a weekday focus schedule from 9:00 to 17:00',
    },
    mascot: { src: 'img/grumpo-lock.webp', width: 649, height: 720 },
  },
  {
    id: 'hold',
    title: 'Get blocked',
    body: "Blocked sites don't load in any browser. Blocked apps close.",
    screenshot: {
      src: 'img/today-limit.webp',
      alt: 'The Today screen with a red banner saying youtube.com is done for today because the daily limit is used up',
    },
    mascot: { src: 'img/grumpo-shield.webp', width: 720, height: 560 },
  },
];

export interface Detail {
  title: string;
  body: string;
}

export const details: readonly Detail[] = [
  { title: 'Daily limits', body: 'A warning at 5 minutes left. Then blocked until midnight.' },
  {
    title: 'Every browser',
    body: 'Sites are blocked system-wide. In Chrome, Brave, Edge and Safari, the tab closes too.',
  },
  {
    title: "Quitting won't help",
    body: 'Quit the app and sites stay blocked. The block lives in a system service.',
  },
  {
    title: "Can't lock you out",
    body: 'Finder, System Settings and the Dock can never be blocked.',
  },
  { title: 'Incognito counts', body: "Private windows count toward limits. They're never saved." },
  { title: 'Fair counting', body: "Idle time doesn't count. Neither does a locked screen." },
];

export interface Question {
  question: string;
  answer: string;
  link?: { label: string; href: string };
}

export const questions: readonly Question[] = [
  {
    question: 'Is it free?',
    answer: 'Yes. Open source, no account, no subscription.',
    link: { label: 'License', href: site.docs.license },
  },
  {
    question: 'Why does it need my password?',
    answer:
      'Blocking sites in every browser means editing a system file. That takes admin rights, once.',
  },
  {
    question: 'Can I just quit it?',
    answer: 'You can. Sites stay blocked anyway. Grumpo stops impulses, not admins.',
    link: { label: 'Every known way around it', href: site.docs.threatModel },
  },
  { question: 'Does it work with Firefox?', answer: "Sites are blocked. Tabs can't be closed." },
  {
    question: 'What about other accounts on my Mac?',
    answer: 'Sites are blocked for everyone. Only you can change the rules.',
  },
  { question: 'Windows or Linux?', answer: 'Not yet. Mac first.' },
  {
    question: 'How do I uninstall it?',
    answer: 'Run the uninstall script, then delete the app.',
    link: { label: 'Uninstall steps', href: site.docs.uninstall },
  },
];

export interface InstallStep {
  title: string;
  body: string;
}

export const installSteps: readonly InstallStep[] = [
  { title: 'Drag to Applications', body: 'Open the download and drag Grumpo in.' },
  {
    title: 'Open it',
    body: 'If macOS blocks it, go to System Settings, Privacy & Security, and click Open Anyway.',
  },
  {
    title: 'Install the service',
    body: 'Enter your password once. This keeps sites blocked even with the app closed.',
  },
  { title: 'Allow browser access', body: 'So Grumpo can see and close blocked tabs.' },
];
