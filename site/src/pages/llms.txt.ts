import type { APIRoute } from 'astro';

import {
  details,
  installSteps,
  networkNote,
  questions,
  terminalInstall,
  tourSteps,
} from '@/config/content';
import { unlockDemo } from '@/config/demo';
import { absolute, installCommand, site } from '@/config/site';

const ENDS_WITH_PUNCTUATION = /[.?!]$/;

const asSentence = (text: string): string => (ENDS_WITH_PUNCTUATION.test(text) ? text : `${text}.`);
const bullet = (title: string, body: string): string => `- ${asSentence(title)} ${body}`;

function overview(): string[] {
  return [
    `# ${site.name}`,
    `> ${site.description}`,
    `${site.name} is a free, open source focus app for macOS ${site.minimumMacOS} or later, built with Rust and Tauri. A root service blocks sites through /etc/hosts in every browser, and the app quits blocked apps, closes blocked tabs and tracks screen time per app and per site. Everything stays on the Mac. ${networkNote}`,
    '## How it works',
    tourSteps.map((step) => bullet(step.title, step.body)).join('\n'),
  ];
}

function unlocking(): string[] {
  const cooldowns = unlockDemo.cooldowns.map((rung) => rung.minutes).join(', ');
  return [
    '## Unlocking',
    `Loosening a rule early takes a cooldown of ${cooldowns} minutes for the first, second, third and later unlocks of the day, then retyping the sentence "${unlockDemo.sentence}" without pasting. An unlock lasts ${unlockDemo.unlockMinutes.join(', ')} minutes, and the count resets at midnight.`,
  ];
}

function detailsSection(): string[] {
  return ['## Details', details.map((detail) => bullet(detail.title, detail.body)).join('\n')];
}

function reference(base: URL | undefined): string[] {
  return [
    '## Install',
    installSteps.map((step, index) => `${index + 1}. ${step.title}. ${step.body}`).join('\n'),
    `${terminalInstall.title}. ${terminalInstall.body} Run \`${installCommand(base)}\`.`,
    '## FAQ',
    questions.map((item) => bullet(item.question, item.answer)).join('\n'),
    '## Links',
    [
      `- [Website](${absolute('', base)})`,
      `- [Download for Mac](${site.download})`,
      `- [Install script](${absolute(site.installScriptPath, base)})`,
      `- [Source code](${site.repo})`,
      `- [Privacy](${site.docs.privacy})`,
      `- [Threat model](${site.docs.threatModel})`,
    ].join('\n'),
  ];
}

export const GET: APIRoute = ({ site: base }) => {
  const body = [...overview(), ...unlocking(), ...detailsSection(), ...reference(base)].join(
    '\n\n',
  );
  return new Response(`${body}\n`, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
};
