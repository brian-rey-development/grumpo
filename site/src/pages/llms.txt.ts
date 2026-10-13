import type { APIRoute } from 'astro';

import { details, installSteps, questions, tourSteps } from '@/config/content';
import { unlockDemo } from '@/config/demo';
import { absolute, site } from '@/config/site';

const bullet = (title: string, body: string): string => `- ${title}. ${body}`;

function overview(): string[] {
  return [
    `# ${site.name}`,
    `> ${site.description}`,
    `${site.name} is a free, open source focus app for macOS ${site.minimumMacOS} or later, built with Rust and Tauri. A root service blocks sites through /etc/hosts in every browser, and the app quits blocked apps, closes blocked tabs and tracks screen time per app and per site. Everything stays on the Mac.`,
    '## How it works',
    tourSteps.map((step) => bullet(step.title, step.body)).join('\n'),
  ];
}

function unlocking(): string[] {
  const cooldowns = unlockDemo.cooldowns.map((rung) => rung.minutes).join(', ');
  return [
    '## Unlocking',
    `Loosening a rule early takes a cooldown of ${cooldowns} minutes for the first, second, third and later unlocks of the day, then retyping the sentence "${unlockDemo.sentence}" without pasting. An unlock lasts ${unlockDemo.unlockMinutes.join(', ')} minutes, and the count resets at midnight.`,
    '## Details',
    details.map((detail) => bullet(detail.title, detail.body)).join('\n'),
  ];
}

function reference(home: string): string[] {
  return [
    '## Install',
    installSteps.map((step, index) => `${index + 1}. ${step.title}. ${step.body}`).join('\n'),
    '## FAQ',
    questions.map((item) => bullet(item.question, item.answer)).join('\n'),
    '## Links',
    [
      `- [Website](${home})`,
      `- [Download for Mac](${site.download})`,
      `- [Source code](${site.repo})`,
      `- [Privacy](${site.docs.privacy})`,
      `- [Threat model](${site.docs.threatModel})`,
    ].join('\n'),
  ];
}

export const GET: APIRoute = ({ site: base }) => {
  const body = [...overview(), ...unlocking(), ...reference(absolute('', base))].join('\n\n');
  return new Response(`${body}\n`, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
};
