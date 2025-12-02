import { create } from "zustand";
import { ToastType } from "../components/ui/Toast";

export interface ToastItem {
  id: string;
  message: string;
  type: ToastType;
}

interface ToastStore {
  toasts: ToastItem[];
  addToast: (message: string, type: ToastType) => void;
  removeToast: (id: string) => void;
  clearAll: () => void;
  // Convenience methods
  success: (message: string) => void;
  error: (message: string) => void;
  warning: (message: string) => void;
  info: (message: string) => void;
}

// Generate unique ID
const generateId = () => `toast-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

export const useToast = create<ToastStore>((set) => ({
  toasts: [],

  addToast: (message, type) => {
    const id = generateId();
    set((state) => ({
      toasts: [...state.toasts, { id, message, type }],
    }));

    // Auto-remove after duration (4s + 300ms for animation)
    setTimeout(() => {
      set((state) => ({
        toasts: state.toasts.filter((t) => t.id !== id),
      }));
    }, 4300);
  },

  removeToast: (id) => {
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    }));
  },

  clearAll: () => {
    set({ toasts: [] });
  },

  // Convenience methods
  success: (message) => {
    const id = generateId();
    set((state) => ({
      toasts: [...state.toasts, { id, message, type: "success" }],
    }));
    setTimeout(() => {
      set((state) => ({
        toasts: state.toasts.filter((t) => t.id !== id),
      }));
    }, 4300);
  },

  error: (message) => {
    const id = generateId();
    set((state) => ({
      toasts: [...state.toasts, { id, message, type: "error" }],
    }));
    setTimeout(() => {
      set((state) => ({
        toasts: state.toasts.filter((t) => t.id !== id),
      }));
    }, 4300);
  },

  warning: (message) => {
    const id = generateId();
    set((state) => ({
      toasts: [...state.toasts, { id, message, type: "warning" }],
    }));
    setTimeout(() => {
      set((state) => ({
        toasts: state.toasts.filter((t) => t.id !== id),
      }));
    }, 4300);
  },

  info: (message) => {
    const id = generateId();
    set((state) => ({
      toasts: [...state.toasts, { id, message, type: "info" }],
    }));
    setTimeout(() => {
      set((state) => ({
        toasts: state.toasts.filter((t) => t.id !== id),
      }));
    }, 4300);
  },
}));

export default useToast;
