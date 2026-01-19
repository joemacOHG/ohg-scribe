<script lang="ts">
    import VocabPresetDropdown from "./VocabPresetDropdown.svelte";
    import { vocabularyStore } from "$lib/stores/vocabulary";

    interface Props {
        boostWords: string[]; // Array of words, not string
        boostWordsInput: string; // The raw input string
        selectedPresets: string[];
        onBoostWordsChange: (words: string[]) => void;
        onBoostWordsInputChange: (value: string) => void;
        onPresetsChange: (presets: string[]) => void;
        onOpenManager?: () => void;
        onOpenExtract?: () => void;
    }

    let {
        boostWords,
        boostWordsInput,
        selectedPresets,
        onBoostWordsChange,
        onBoostWordsInputChange,
        onPresetsChange,
        onOpenManager,
        onOpenExtract,
    }: Props = $props();

    let isExpanded = $state(false);
    const PREVIEW_COUNT = 10;

    // Get all preset terms merged together
    function getPresetTerms(): string[] {
        return vocabularyStore.getTermsForPresets(selectedPresets);
    }

    // All words combined: user words + preset terms
    function getAllWords(): string[] {
        const presetTerms = getPresetTerms();
        return [...boostWords, ...presetTerms];
    }

    // Handle adding word when user presses Enter or comma
    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === "Enter" || e.key === ",") {
            e.preventDefault();
            addCurrentWord();
        }
    }

    function handleInputChange(e: Event) {
        const value = (e.target as HTMLInputElement).value;
        onBoostWordsInputChange(value);
    }

    function addCurrentWord() {
        const trimmed = boostWordsInput.trim().replace(/,/g, "");
        if (trimmed && !boostWords.includes(trimmed)) {
            onBoostWordsChange([...boostWords, trimmed]);
            onBoostWordsInputChange("");
        }
    }

    function removeWord(word: string) {
        onBoostWordsChange(boostWords.filter((w) => w !== word));
    }

    function removePreset(id: string) {
        onPresetsChange(selectedPresets.filter((p) => p !== id));
    }

    function getPresetName(id: string): string {
        const preset = vocabularyStore.getPreset(id);
        return preset?.name ?? id;
    }

    function getPresetTermCount(id: string): number {
        const preset = vocabularyStore.getPreset(id);
        return preset?.terms.length ?? 0;
    }

    // Words to display based on expanded/collapsed state
    function getVisibleWords(): string[] {
        const all = getAllWords();
        if (isExpanded || all.length <= PREVIEW_COUNT) {
            return all;
        }
        return all.slice(0, PREVIEW_COUNT);
    }

    function getRemainingCount(): number {
        const all = getAllWords();
        return Math.max(0, all.length - PREVIEW_COUNT);
    }

    $effect(() => {
        // Re-render when store updates
        $vocabularyStore;
    });
</script>

<div class="section">
    <div class="section-header">
        <div class="section-icon">
            <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke-width="2"
                stroke-linecap="round"
            >
                <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
                <path
                    d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"
                />
            </svg>
        </div>
        <div>
            <div class="section-title">Vocabulary</div>
            <div class="section-subtitle">
                Improve recognition of domain terms
            </div>
        </div>
    </div>

    <div class="section-content">
        <!-- Input Row -->
        <div class="vocab-row">
            <div class="vocab-input-wrap">
                <label class="input-label" for="boost-words">Add words</label>
                <input
                    type="text"
                    id="boost-words"
                    class="new-input"
                    placeholder="Type a term and press Enter"
                    value={boostWordsInput}
                    oninput={handleInputChange}
                    onkeydown={handleKeyDown}
                />
            </div>
            <VocabPresetDropdown
                {selectedPresets}
                {onPresetsChange}
                {onOpenManager}
            />
        </div>

        <!-- Selected Presets -->
        {#if selectedPresets.length > 0}
            <div class="selected-presets">
                {#each selectedPresets as presetId}
                    <span class="preset-tag">
                        <span class="preset-name"
                            >{getPresetName(presetId)}</span
                        >
                        <span class="preset-count"
                            >{getPresetTermCount(presetId)}</span
                        >
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <svg
                            class="remove"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke-width="2"
                            onclick={() => removePreset(presetId)}
                        >
                            <line x1="18" y1="6" x2="6" y2="18" />
                            <line x1="6" y1="6" x2="18" y2="18" />
                        </svg>
                    </span>
                {/each}
            </div>
        {/if}

        <!-- Word Tags -->
        {#if getAllWords().length > 0}
            <div class="word-tags-container">
                <div class="word-tags-header">
                    <span class="word-count"
                        >{getAllWords().length} words will boost recognition</span
                    >
                    {#if getAllWords().length > PREVIEW_COUNT}
                        <button
                            class="expand-btn"
                            onclick={() => (isExpanded = !isExpanded)}
                        >
                            {isExpanded
                                ? "Show less"
                                : `Show all ${getAllWords().length}`}
                        </button>
                    {/if}
                </div>
                <div class="word-tags" class:expanded={isExpanded}>
                    {#each getVisibleWords() as word}
                        <span
                            class="word-tag"
                            class:user-word={boostWords.includes(word)}
                        >
                            {word}
                            {#if boostWords.includes(word)}
                                <!-- svelte-ignore a11y_click_events_have_key_events -->
                                <!-- svelte-ignore a11y_no_static_element_interactions -->
                                <svg
                                    class="remove"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke-width="2"
                                    onclick={() => removeWord(word)}
                                >
                                    <line x1="18" y1="6" x2="6" y2="18" />
                                    <line x1="6" y1="6" x2="18" y2="18" />
                                </svg>
                            {/if}
                        </span>
                    {/each}
                    {#if !isExpanded && getRemainingCount() > 0}
                        <span class="more-indicator"
                            >+{getRemainingCount()} more</span
                        >
                    {/if}
                </div>
            </div>
        {:else}
            <div class="inline-help">
                <svg viewBox="0 0 24 24" fill="none" stroke-width="2">
                    <circle cx="12" cy="12" r="10" />
                    <path d="M12 16v-4" />
                    <path d="M12 8h.01" />
                </svg>
                <p>
                    Type custom terms above or select disease areas to auto-load
                    vocabulary. Terms improve recognition of drug names,
                    acronyms, and clinical terms.
                </p>
            </div>
        {/if}

        <!-- Extract from Document link -->
        <button class="extract-link" onclick={onOpenExtract}>
            <svg viewBox="0 0 24 24" fill="none" stroke-width="2">
                <path
                    d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
                />
                <polyline points="14 2 14 8 20 8" />
                <path d="M9 15h6" />
                <path d="M12 18v-6" />
            </svg>
            Extract terms from a document
        </button>
    </div>
</div>

<style>
    .section {
        padding: 16px 20px;
        border-bottom: 1px solid var(--gray-100, #f9fafb);
    }

    .section:last-child {
        border-bottom: none;
    }

    .section-header {
        display: flex;
        align-items: center;
        gap: 10px;
        margin-bottom: 12px;
    }

    .section-icon {
        width: 28px;
        height: 28px;
        background: var(--lavender, #f0ebf5);
        border-radius: 6px;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    .section-icon svg {
        width: 16px;
        height: 16px;
        stroke: var(--purple, #6b2d7b);
    }

    .section-title {
        font-size: 13px;
        font-weight: 600;
        color: var(--navy, #1a2b4a);
    }

    .section-subtitle {
        font-size: 11px;
        color: var(--gray-400, #9ca3af);
        margin-top: 2px;
    }

    .section-content {
        margin-left: 38px;
    }

    .vocab-row {
        display: flex;
        gap: 8px;
        align-items: flex-end;
    }

    .vocab-input-wrap {
        flex: 1;
    }

    .input-label {
        font-size: 11px;
        color: var(--gray-500, #6b7280);
        margin-bottom: 4px;
        display: block;
    }

    .new-input {
        width: 100%;
        padding: 8px 12px;
        border: 1px solid var(--gray-200, #e5e7eb);
        border-radius: 8px;
        font-size: 13px;
        color: var(--navy, #1a2b4a);
        background: var(--gray-100, #f9fafb);
        transition: all 0.2s;
    }

    .new-input:focus {
        outline: none;
        border-color: var(--magenta, #e91388);
        background: white;
        box-shadow: 0 0 0 3px rgba(233, 19, 136, 0.1);
    }

    .new-input::placeholder {
        color: var(--gray-400, #9ca3af);
    }

    /* Selected Presets */
    .selected-presets {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        margin-top: 10px;
    }

    .preset-tag {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 4px 8px 4px 10px;
        background: linear-gradient(
            135deg,
            rgba(233, 19, 136, 0.1) 0%,
            rgba(107, 45, 123, 0.1) 100%
        );
        border: 1px solid rgba(233, 19, 136, 0.3);
        border-radius: 6px;
        font-size: 12px;
        color: var(--magenta, #e91388);
    }

    .preset-name {
        font-weight: 500;
    }

    .preset-count {
        font-size: 10px;
        background: rgba(233, 19, 136, 0.15);
        padding: 1px 5px;
        border-radius: 4px;
    }

    .preset-tag .remove {
        width: 14px;
        height: 14px;
        stroke: var(--magenta, #e91388);
        cursor: pointer;
        opacity: 0.6;
        transition: opacity 0.15s;
    }

    .preset-tag .remove:hover {
        opacity: 1;
    }

    /* Word Tags */
    .word-tags-container {
        margin-top: 12px;
        background: var(--gray-100, #f9fafb);
        border-radius: 8px;
        padding: 10px 12px;
    }

    .word-tags-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;
    }

    .word-count {
        font-size: 11px;
        color: var(--gray-500, #6b7280);
    }

    .expand-btn {
        font-size: 11px;
        color: var(--purple, #6b2d7b);
        background: none;
        border: none;
        cursor: pointer;
        padding: 2px 6px;
        border-radius: 4px;
    }

    .expand-btn:hover {
        background: var(--lavender, #f0ebf5);
    }

    .word-tags {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
    }

    .word-tag {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        padding: 4px 8px;
        background: var(--white, #ffffff);
        border: 1px solid var(--gray-200, #e5e7eb);
        border-radius: 5px;
        font-size: 12px;
        color: var(--gray-600, #4b5563);
    }

    .word-tag.user-word {
        background: var(--lavender-light, #f8f5fa);
        border-color: var(--lavender-dark, #e8e0f0);
        color: var(--purple, #6b2d7b);
    }

    .word-tag .remove {
        width: 12px;
        height: 12px;
        stroke: var(--gray-400, #9ca3af);
        cursor: pointer;
        transition: stroke 0.15s;
    }

    .word-tag.user-word .remove {
        stroke: var(--purple, #6b2d7b);
        opacity: 0.6;
    }

    .word-tag .remove:hover {
        stroke: var(--error, #ef4444);
    }

    .more-indicator {
        display: inline-flex;
        align-items: center;
        padding: 4px 8px;
        background: var(--lavender, #f0ebf5);
        border-radius: 5px;
        font-size: 11px;
        color: var(--purple, #6b2d7b);
        font-weight: 500;
    }

    /* Help text */
    .inline-help {
        display: flex;
        align-items: flex-start;
        gap: 8px;
        padding: 10px 12px;
        background: var(--lavender-light, #f8f5fa);
        border-radius: 8px;
        margin-top: 12px;
    }

    .inline-help svg {
        width: 14px;
        height: 14px;
        stroke: var(--purple, #6b2d7b);
        flex-shrink: 0;
        margin-top: 1px;
    }

    .inline-help p {
        font-size: 11px;
        color: var(--gray-500, #6b7280);
        line-height: 1.4;
    }

    /* Extract link */
    .extract-link {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        margin-top: 12px;
        padding: 8px 12px;
        background: transparent;
        border: 1px dashed var(--gray-300, #d1d5db);
        border-radius: 8px;
        font-size: 12px;
        color: var(--gray-500, #6b7280);
        cursor: pointer;
        transition: all 0.2s;
        width: 100%;
        justify-content: center;
    }

    .extract-link:hover {
        border-color: var(--magenta, #e91388);
        color: var(--magenta, #e91388);
        background: rgba(233, 19, 136, 0.03);
    }

    .extract-link svg {
        width: 14px;
        height: 14px;
        stroke: currentColor;
    }
</style>
