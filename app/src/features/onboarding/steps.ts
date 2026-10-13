export const STEPS = ['welcome', 'how', 'private', 'service', 'permissions', 'done'] as const;

export type StepId = (typeof STEPS)[number];

export type Destination = 'rules' | 'today';

export type PoseName = 'wave' | 'desk' | 'shield' | 'lock' | 'approve' | 'crossed';

export const STEP_POSES = {
  welcome: 'wave',
  how: 'desk',
  private: 'shield',
  service: 'lock',
  permissions: 'crossed',
  done: 'approve',
} as const satisfies Record<StepId, PoseName>;
