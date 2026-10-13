import type { Rules } from '@/ipc/generated/Rules';

export interface EditorProps {
  rules: Rules;
  edit: (change: (rules: Rules) => Rules) => void;
  readOnly: boolean;
}
