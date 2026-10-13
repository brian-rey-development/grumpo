import { AnimatePresence, motion } from 'motion/react';

import { Section } from '@/components/Section/Section';
import { serviceCopy } from '@/copy/service';
import { tween, useMotionTransition } from '@/design/motion';
import { explainCopy } from '@/features/explain/copy';
import { ExplainedRow } from '@/features/explain/ExplainedRow';
import { StatusBadge } from '@/features/explain/StatusBadge';
import { WhyDisclosure } from '@/features/explain/WhyDisclosure';
import {
  InstallButton,
  InstallFallback,
  InstallProgress,
} from '@/features/settings/InstallControls';
import { type InstallableReason, offersInstall } from '@/features/settings/serviceFix';
import { type ServiceInstall, useServiceInstall } from '@/features/settings/useServiceInstall';
import type { FocusStatus } from '@/ipc/generated/FocusStatus';
import { useFocusStatus } from '@/ipc/queries';

import { onboardingCopy } from '../copy';
import { Pose } from '../Pose';
import { STEP_POSES } from '../steps';
import { StepFrame } from '../StepFrame';

import styles from '../Step.module.css';

const copy = onboardingCopy.service;
const FADED = { opacity: 0 } as const;

function installableReason(status: FocusStatus | undefined): InstallableReason | null {
  if (status?.kind !== 'unavailable') return null;
  return offersInstall(status.reason) ? status.reason : null;
}

function otherReason(status: FocusStatus | undefined): string | null {
  if (status?.kind !== 'unavailable' || offersInstall(status.reason)) return null;
  return serviceCopy.reasons[status.reason];
}

interface InstallSlotProps {
  connected: boolean;
  reason: InstallableReason | null;
  install: ServiceInstall;
}

/** One fixed-width place that holds the button and then the result, so the text beside it never reflows. */
function InstallSlot({ connected, reason, install }: InstallSlotProps) {
  const fade = useMotionTransition(tween.fast);
  const showsButton = !connected && reason !== null && !install.unavailable;

  return (
    <span className={styles.slot}>
      <AnimatePresence mode="wait" initial={false}>
        {connected ? (
          <motion.span key="running" initial={FADED} animate={{ opacity: 1 }} transition={fade}>
            <StatusBadge tone="ok" label={copy.running} />
          </motion.span>
        ) : null}
        {showsButton ? (
          <motion.span
            key="install"
            initial={FADED}
            animate={{ opacity: 1 }}
            exit={FADED}
            transition={fade}
          >
            <InstallButton reason={reason} install={install} />
          </motion.span>
        ) : null}
      </AnimatePresence>
    </span>
  );
}

export function ServiceStep() {
  const status = useFocusStatus().data;
  const install = useServiceInstall();
  const reason = installableReason(status);
  const note = otherReason(status);

  return (
    <StepFrame hero={<Pose name={STEP_POSES.service} />} title={copy.title} lede={copy.lede}>
      <Section as="list">
        <ExplainedRow
          label={copy.label}
          detail={explainCopy.service.purpose}
          trailing={
            <InstallSlot
              connected={status?.kind === 'connected'}
              reason={reason}
              install={install}
            />
          }
        >
          {note === null ? null : <span className={styles.note}>{note}</span>}
          <WhyDisclosure explanation={explainCopy.service} />
        </ExplainedRow>
      </Section>
      {reason === null ? null : <InstallProgress reason={reason} install={install} />}
      <InstallFallback install={install} intro={copy.runCommand} />
    </StepFrame>
  );
}
