<script lang="ts">
  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSave: (assemblyaiKey: string, openaiKey: string) => void;
    currentAssemblyAIKey?: string;
    currentOpenAIKey?: string;
  }

  let {
    isOpen,
    onClose,
    onSave,
    currentAssemblyAIKey = "",
    currentOpenAIKey = "",
  }: Props = $props();

  let assemblyaiKey = $state("");
  let openaiKey = $state("");
  let showKey = $state(false);
  let showOpenAIKey = $state(false);

  $effect(() => {
    if (isOpen) {
      assemblyaiKey = currentAssemblyAIKey;
      openaiKey = currentOpenAIKey;
    }
  });

  function handleSave() {
    if (assemblyaiKey.trim()) {
      onSave(assemblyaiKey.trim(), openaiKey.trim());
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    } else if (e.key === "Enter") {
      handleSave();
    }
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="modal-overlay"
    onclick={onClose}
    onkeydown={handleKeydown}
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-labelledby="settings-title"
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="modal" onclick={(e) => e.stopPropagation()} role="document">
      <div class="modal-header">
        <h2 id="settings-title">Settings</h2>
        <button class="close-btn" onclick={onClose} aria-label="Close">×</button
        >
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label for="api-key">AssemblyAI API Key</label>
          <div class="input-wrapper">
            <input
              type={showKey ? "text" : "password"}
              id="api-key"
              bind:value={assemblyaiKey}
              placeholder="Enter your API key"
            />
            <button
              class="toggle-visibility"
              onclick={() => (showKey = !showKey)}
              type="button"
              aria-label={showKey ? "Hide API key" : "Show API key"}
            >
              {showKey ? "🙈" : "👁️"}
            </button>
          </div>
          <p class="help-text">
            For transcription. Get your key from <a
              href="https://www.assemblyai.com"
              target="_blank"
              rel="noopener">assemblyai.com</a
            >
          </p>
        </div>

        <div class="form-group" style="margin-top: 20px;">
          <label for="openai-key"
            >OpenAI API Key <span class="optional">(optional)</span></label
          >
          <div class="input-wrapper">
            <input
              type={showOpenAIKey ? "text" : "password"}
              id="openai-key"
              bind:value={openaiKey}
              placeholder="sk-..."
            />
            <button
              class="toggle-visibility"
              onclick={() => (showOpenAIKey = !showOpenAIKey)}
              type="button"
              aria-label={showOpenAIKey ? "Hide API key" : "Show API key"}
            >
              {showOpenAIKey ? "🙈" : "👁️"}
            </button>
          </div>
          <p class="help-text">
            For extracting vocabulary from documents. Get your key from <a
              href="https://platform.openai.com/api-keys"
              target="_blank"
              rel="noopener">platform.openai.com</a
            >
          </p>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-cancel" onclick={onClose}>Cancel</button>
        <button
          class="btn-save"
          onclick={handleSave}
          disabled={!assemblyaiKey.trim()}
        >
          Save
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(26, 43, 74, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--white, #ffffff);
    border-radius: 16px;
    width: 90%;
    max-width: 500px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    border-bottom: 1px solid var(--lavender-dark, #e8e0f0);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--navy, #1a2b4a);
  }

  .close-btn {
    width: 32px;
    height: 32px;
    background: var(--lavender, #f0ebf5);
    border: none;
    border-radius: 8px;
    font-size: 18px;
    color: var(--gray-600, #4b5563);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .close-btn:hover {
    background: var(--lavender-dark, #e8e0f0);
    color: var(--navy, #1a2b4a);
  }

  .modal-body {
    padding: 24px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-group label {
    font-size: 14px;
    font-weight: 500;
    color: var(--navy, #1a2b4a);
  }

  .input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  .input-wrapper input {
    flex: 1;
    padding: 10px 40px 10px 12px;
    border: 1px solid var(--gray-200, #e5e7eb);
    border-radius: 8px;
    font-size: 14px;
    color: var(--navy, #1a2b4a);
  }

  .input-wrapper input:focus {
    outline: none;
    border-color: var(--magenta, #e91388);
    box-shadow: 0 0 0 3px rgba(233, 19, 136, 0.1);
  }

  .toggle-visibility {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px;
    font-size: 16px;
  }

  .help-text {
    font-size: 12px;
    color: var(--gray-600, #4b5563);
    margin: 0;
  }

  .help-text a {
    color: var(--magenta, #e91388);
    text-decoration: none;
  }

  .help-text a:hover {
    text-decoration: underline;
  }

  .optional {
    font-weight: 400;
    color: var(--gray-400, #9ca3af);
    font-size: 12px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 24px;
    border-top: 1px solid var(--lavender-dark, #e8e0f0);
  }

  .btn-cancel,
  .btn-save {
    padding: 10px 20px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-cancel {
    background: var(--white, #ffffff);
    border: 1px solid var(--magenta, #e91388);
    color: var(--magenta, #e91388);
  }

  .btn-cancel:hover {
    background: rgba(233, 19, 136, 0.05);
  }

  .btn-save {
    background: linear-gradient(
      135deg,
      var(--magenta, #e91388) 0%,
      var(--purple, #6b2d7b) 100%
    );
    border: none;
    color: white;
  }

  .btn-save:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(233, 19, 136, 0.3);
  }

  .btn-save:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
