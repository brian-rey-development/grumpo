import { useCallback, useRef, useState } from 'react';

export interface ToastMessage {
  id: number;
  text: string;
}

interface ToastController {
  toast: ToastMessage | null;
  show: (text: string) => void;
  dismiss: () => void;
}

/** Holds the one toast on screen. A newer message replaces the current one. */
export function useToast(): ToastController {
  const [toast, setToast] = useState<ToastMessage | null>(null);
  const nextId = useRef(0);

  const show = useCallback((text: string) => {
    nextId.current += 1;
    setToast({ id: nextId.current, text });
  }, []);
  const dismiss = useCallback(() => {
    setToast(null);
  }, []);

  return { toast, show, dismiss };
}
