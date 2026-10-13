import {
  type ClipboardEvent,
  type DragEvent,
  type InputHTMLAttributes,
  type Ref,
  useCallback,
  useId,
  useRef,
} from 'react';

import { LOCKED_ATTRIBUTES, usePasteGuard } from '@/components/TextField/usePasteGuard';
import { cx } from '@/lib/classNames';

import styles from './TextField.module.css';

interface TextFieldProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> {
  label: string;
  hint?: string | undefined;
  error?: string | undefined;
  preventPaste?: boolean;
  ref?: Ref<HTMLInputElement>;
}

function assignRef(ref: Ref<HTMLInputElement> | undefined, element: HTMLInputElement | null): void {
  if (typeof ref === 'function') ref(element);
  else if (ref) ref.current = element;
}

interface FieldMessageProps {
  id: string;
  isError: boolean;
  children: string;
}

function FieldMessage({ id, isError, children }: FieldMessageProps) {
  return (
    <p
      id={id}
      className={isError ? styles.error : styles.hint}
      role={isError ? 'alert' : undefined}
    >
      {children}
    </p>
  );
}

export function TextField({
  label,
  hint,
  error,
  preventPaste = false,
  id,
  className,
  onPaste,
  onDrop,
  ref,
  ...rest
}: TextFieldProps) {
  const generatedId = useId();
  const inputId = id ?? generatedId;
  const messageId = `${inputId}-message`;
  const message = error ?? hint;
  const inputRef = useRef<HTMLInputElement | null>(null);
  const setRefs = useCallback(
    (element: HTMLInputElement | null) => {
      inputRef.current = element;
      assignRef(ref, element);
    },
    [ref],
  );
  usePasteGuard(inputRef, preventPaste);

  function blockPaste(event: ClipboardEvent<HTMLInputElement>): void {
    if (preventPaste) event.preventDefault();
    onPaste?.(event);
  }

  function blockDrop(event: DragEvent<HTMLInputElement>): void {
    if (preventPaste) event.preventDefault();
    onDrop?.(event);
  }

  return (
    <div className={styles.field}>
      <label htmlFor={inputId} className={styles.label}>
        {label}
      </label>
      <input
        {...rest}
        {...(preventPaste ? LOCKED_ATTRIBUTES : {})}
        ref={setRefs}
        id={inputId}
        className={cx(styles.input, className)}
        aria-invalid={error === undefined ? undefined : true}
        aria-describedby={message === undefined ? undefined : messageId}
        onPaste={blockPaste}
        onDrop={blockDrop}
      />
      {message === undefined ? null : (
        <FieldMessage id={messageId} isError={error !== undefined}>
          {message}
        </FieldMessage>
      )}
    </div>
  );
}
