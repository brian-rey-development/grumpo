import { createContext, useContext, useEffect, useId } from 'react';

interface ModalStackValue {
  topId: string | null;
  depth: number;
  register: (id: string) => () => void;
}

export const ModalStackContext = createContext<ModalStackValue | null>(null);

function useModalStack(): ModalStackValue {
  const value = useContext(ModalStackContext);
  if (value === null) {
    throw new Error('Modal components must be used inside a ModalStackProvider');
  }
  return value;
}

export function useIsModalOpen(): boolean {
  return useModalStack().depth > 0;
}

/** Keeps `id` on the modal stack while `present`, which also makes the app behind it inert. */
export function useModalRegistration(present: boolean): { isTop: boolean } {
  const id = useId();
  const { topId, register } = useModalStack();

  useEffect(() => {
    if (!present) return undefined;
    return register(id);
  }, [present, register, id]);

  return { isTop: topId === id };
}
