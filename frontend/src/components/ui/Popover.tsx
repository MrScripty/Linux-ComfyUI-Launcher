import {
  useEffect,
  useId,
  useRef,
  type MouseEventHandler,
  type ReactNode,
  type RefCallback,
  type RefObject,
} from 'react';
import { AnimatePresence, motion, useReducedMotion } from 'framer-motion';
import { registerOverlayEscapeLayer } from './OverlayEscapeStack';

const FOCUSABLE_SELECTOR = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
].join(',');

type PopoverEntry = {
  id: string;
  panelRef: RefObject<HTMLDivElement | null>;
  restoreFocus: HTMLElement | null;
  triggerRef: RefObject<HTMLButtonElement | null>;
};

const popoverStack: PopoverEntry[] = [];

function getTopPopover(): PopoverEntry | undefined {
  for (let index = popoverStack.length - 1; index >= 0; index -= 1) {
    const entry = popoverStack[index];
    if (entry?.panelRef.current?.isConnected) {
      return entry;
    }
  }
  return undefined;
}

function focusPopover(
  panel: HTMLDivElement,
  initialFocusRef?: RefObject<HTMLElement | null>
): void {
  const requestedTarget = initialFocusRef?.current;
  if (requestedTarget?.isConnected && panel.contains(requestedTarget)) {
    requestedTarget.focus();
    return;
  }

  const firstFocusable = panel.querySelector<HTMLElement>(FOCUSABLE_SELECTOR);
  (firstFocusable ?? panel).focus();
}

export interface PopoverTriggerProps {
  'aria-controls': string;
  'aria-expanded': boolean;
  'aria-haspopup': 'dialog';
  onClick: MouseEventHandler<HTMLButtonElement>;
  ref: RefCallback<HTMLButtonElement>;
}

export interface PopoverProps {
  children: ReactNode;
  contentClassName?: string;
  initialFocusRef?: RefObject<HTMLElement | null>;
  isOpen: boolean;
  label: string;
  onOpenChange: (isOpen: boolean) => void;
  rootClassName?: string;
  trigger: (props: PopoverTriggerProps) => ReactNode;
}

/** Owns non-modal popup relationships, focus entry/return, and dismissal. */
export function Popover({
  children,
  contentClassName = '',
  initialFocusRef,
  isOpen,
  label,
  onOpenChange,
  rootClassName = 'relative',
  trigger,
}: PopoverProps) {
  const generatedId = useId();
  const entryId = useRef(`popover-${generatedId}`);
  const contentId = `${entryId.current}-content`;
  const rootRef = useRef<HTMLDivElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const shouldReduceMotion = useReducedMotion();
  const entryOffset = shouldReduceMotion ? 0 : -6;
  const onOpenChangeRef = useRef(onOpenChange);
  onOpenChangeRef.current = onOpenChange;

  useEffect(() => {
    if (!isOpen) {
      return undefined;
    }

    const entry: PopoverEntry = {
      id: entryId.current,
      panelRef,
      restoreFocus: document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null,
      triggerRef,
    };
    popoverStack.push(entry);
    const unregisterEscapeLayer = registerOverlayEscapeLayer(entry.id, () => {
      onOpenChangeRef.current(false);
    });
    const focusTimer = window.setTimeout(() => {
      if (getTopPopover()?.id === entry.id && panelRef.current) {
        focusPopover(panelRef.current, initialFocusRef);
      }
    }, 0);

    const handleOutsideInteraction = (event: globalThis.MouseEvent | TouchEvent) => {
      if (
        event.target instanceof Node &&
        rootRef.current &&
        !rootRef.current.contains(event.target)
      ) {
        onOpenChangeRef.current(false);
      }
    };

    document.addEventListener('mousedown', handleOutsideInteraction, true);
    document.addEventListener('touchstart', handleOutsideInteraction, true);
    return () => {
      window.clearTimeout(focusTimer);
      document.removeEventListener('mousedown', handleOutsideInteraction, true);
      document.removeEventListener('touchstart', handleOutsideInteraction, true);
      unregisterEscapeLayer();

      const entryIndex = popoverStack.findIndex((candidate) => candidate.id === entry.id);
      const wasTopPopover = entryIndex === popoverStack.length - 1;
      if (entryIndex >= 0) {
        popoverStack.splice(entryIndex, 1);
      }
      if (wasTopPopover) {
        const focusTarget = entry.restoreFocus?.isConnected
          ? entry.restoreFocus
          : entry.triggerRef.current;
        focusTarget?.focus();
      }
    };
  }, [initialFocusRef, isOpen]);

  const assignTriggerRef: RefCallback<HTMLButtonElement> = (element) => {
    triggerRef.current = element;
  };
  const triggerProps: PopoverTriggerProps = {
    'aria-controls': contentId,
    'aria-expanded': isOpen,
    'aria-haspopup': 'dialog',
    onClick: () => onOpenChange(!isOpen),
    ref: assignTriggerRef,
  };

  return (
    <div ref={rootRef} className={rootClassName}>
      {trigger(triggerProps)}
      <AnimatePresence>
        {isOpen && (
          <motion.div
            ref={panelRef}
            id={contentId}
            role="dialog"
            aria-label={label}
            className={contentClassName}
            initial={{ opacity: 0, y: entryOffset }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: entryOffset }}
            transition={{ duration: 0.16 }}
            tabIndex={-1}
          >
            {children}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
