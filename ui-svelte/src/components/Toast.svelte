<script lang="ts">
  import { app } from '../lib/stores/app.svelte'
</script>

{#if app.toast}
  <div
    class="toast"
    class:error={app.toast.isError}
    role="alert"
  >
    <span>{app.toast.message}</span>
    <button class="close-btn" onclick={() => app.dismissToast()} aria-label="Close">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    bottom: 24px;
    right: 24px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 8px;
    background: #1a3a2a;
    color: #4ade80;
    border: 1px solid #2d5a3d;
    font-size: 0.875rem;
    z-index: 1000;
    animation: slideIn 0.2s ease;
  }

  .toast.error {
    background: #3a1a1a;
    color: #f87171;
    border-color: #5a2d2d;
  }

  .close-btn {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 2px;
    display: flex;
    opacity: 0.7;
  }

  .close-btn:hover {
    opacity: 1;
  }

  .close-btn svg {
    width: 16px;
    height: 16px;
  }

  @keyframes slideIn {
    from {
      transform: translateY(16px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }
</style>
