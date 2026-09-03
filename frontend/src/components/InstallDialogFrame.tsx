import { useId, useRef, type ReactNode } from 'react';
import { X } from 'lucide-react';
import { ModalDialog } from './ui';

interface InstallDialogFrameProps {
  children: ReactNode;
  isOpen: boolean;
  isPageMode: boolean;
  onClose: () => void;
  title: string;
}

export function InstallDialogFrame({
  children,
  isOpen,
  isPageMode,
  onClose,
  title,
}: InstallDialogFrameProps) {
  const titleId = useId();
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  if (!isOpen) {
    return null;
  }

  if (isPageMode) {
    return (
      <div className="w-full h-full min-h-0 flex flex-col">
        {children}
      </div>
    );
  }

  return (
    <ModalDialog
      ariaLabelledBy={titleId}
      backdropClassName="bg-black/70"
      contentClassName="w-full max-w-3xl max-h-[80vh] min-h-0 flex flex-col"
      initialFocusRef={closeButtonRef}
      isOpen={true}
      onClose={onClose}
    >
      <div className="flex items-center justify-between p-4 border-b border-[hsl(var(--border-default))]">
        <div className="flex items-center gap-3">
          <h2 id={titleId} className="text-xl font-semibold text-[hsl(var(--text-primary))]">
            {title}
          </h2>
        </div>
        <div className="flex items-center gap-2">
          <button
            ref={closeButtonRef}
            onClick={onClose}
            className="p-1 rounded hover:bg-[hsl(var(--surface-interactive-hover))] transition-colors"
            aria-label="Close install dialog"
          >
            <X size={20} className="text-[hsl(var(--text-muted))]" />
          </button>
        </div>
      </div>
      {children}
    </ModalDialog>
  );
}
