interface DeleteConfirmModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  isLoading: boolean;
  title: string;
  message: string;
  warning?: string;
  confirmText?: string;
  loadingText?: string;
}

export function DeleteConfirmModal({
  isOpen,
  onClose,
  onConfirm,
  isLoading,
  title,
  message,
  warning = "This action cannot be undone.",
  confirmText = "Delete",
  loadingText = "Deleting...",
}: DeleteConfirmModalProps) {
  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal-card delete-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <span className="modal-icon">⚠️</span>
          <h2>{title}</h2>
        </div>
        <div className="modal-body">
          <p>{message}</p>
          <p className="modal-warning">{warning}</p>
        </div>
        <div className="modal-actions">
          <button
            type="button"
            className="btn-secondary"
            onClick={onClose}
            disabled={isLoading}
          >
            Cancel
          </button>
          <button
            type="button"
            className="btn-danger"
            onClick={onConfirm}
            disabled={isLoading}
          >
            {isLoading ? loadingText : confirmText}
          </button>
        </div>
      </div>
    </div>
  );
}
