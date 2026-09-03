type EscapeLayer = {
  id: string;
  onEscape: () => void;
};

const escapeLayers: EscapeLayer[] = [];

function handleDocumentKeyDown(event: KeyboardEvent): void {
  if (event.key !== 'Escape') {
    return;
  }

  const topLayer = escapeLayers.at(-1);
  if (!topLayer) {
    return;
  }

  event.preventDefault();
  event.stopPropagation();
  event.stopImmediatePropagation();
  topLayer.onEscape();
}

/** Arbitrates Escape across modal and non-modal overlay Modules. */
export function registerOverlayEscapeLayer(id: string, onEscape: () => void): () => void {
  escapeLayers.push({ id, onEscape });
  if (escapeLayers.length === 1) {
    document.addEventListener('keydown', handleDocumentKeyDown, true);
  }

  return () => {
    const layerIndex = escapeLayers.findIndex((layer) => layer.id === id);
    if (layerIndex >= 0) {
      escapeLayers.splice(layerIndex, 1);
    }
    if (escapeLayers.length === 0) {
      document.removeEventListener('keydown', handleDocumentKeyDown, true);
    }
  };
}
