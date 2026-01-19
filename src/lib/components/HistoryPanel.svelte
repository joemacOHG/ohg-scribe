<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import {
        getHistoryList,
        deleteHistoryEntry,
        type HistorySummary,
    } from "$lib/services/history";

    interface Props {
        isOpen: boolean;
        onClose: () => void;
    }

    let { isOpen, onClose }: Props = $props();

    let historyList: HistorySummary[] = $state([]);
    let loading = $state(false);
    let error = $state<string | null>(null);

    onMount(() => {
        if (isOpen) {
            loadHistory();
        }
    });

    $effect(() => {
        if (isOpen) {
            loadHistory();
        }
    });

    async function loadHistory() {
        loading = true;
        error = null;
        try {
            historyList = await getHistoryList();
        } catch (e) {
            error = `Failed to load history: ${e instanceof Error ? e.message : String(e)}`;
            console.error(error);
        } finally {
            loading = false;
        }
    }

    function openTranscript(id: string) {
        onClose(); // Close the modal
        goto(`/transcript/${id}`); // Navigate to full page
    }

    async function deleteEntry(id: string, e: Event) {
        e.stopPropagation(); // Don't trigger the parent click

        if (!confirm("Are you sure you want to delete this transcription?")) {
            return;
        }

        try {
            await deleteHistoryEntry(id);
            historyList = historyList.filter((h) => h.id !== id);
        } catch (e) {
            error = `Failed to delete: ${e instanceof Error ? e.message : String(e)}`;
        }
    }

    function formatDate(isoString: string): string {
        const date = new Date(isoString);
        return date.toLocaleDateString("en-US", {
            month: "short",
            day: "numeric",
            year: "numeric",
            hour: "numeric",
            minute: "2-digit",
        });
    }

    // Clean up filename for display
    function cleanTitle(filename: string): string {
        return filename
            .replace(/\.[^/.]+$/, "") // Remove extension
            .replace(/_/g, " ") // Underscores to spaces
            .replace(/\s+/g, " ") // Normalize spaces
            .trim();
    }
</script>

{#if isOpen}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
        class="modal-overlay"
        role="dialog"
        aria-modal="true"
        aria-labelledby="history-title"
        onclick={onClose}
    >
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div class="modal" role="document" onclick={(e) => e.stopPropagation()}>
            <div class="modal-header">
                <h2 id="history-title">Transcription History</h2>
                <button
                    class="close-btn"
                    onclick={onClose}
                    aria-label="Close history"
                >
                    ×
                </button>
            </div>

            <div class="modal-body">
                {#if error}
                    <div class="error-message">{error}</div>
                {/if}

                {#if loading}
                    <div class="loading">
                        <div class="spinner"></div>
                        <p>Loading history...</p>
                    </div>
                {:else if historyList.length === 0}
                    <div class="empty-state">
                        <p>No transcriptions yet.</p>
                        <p>Drop a video or audio file to get started!</p>
                    </div>
                {:else}
                    <div class="history-list">
                        {#each historyList as item (item.id)}
                            <!-- svelte-ignore a11y_no_static_element_interactions -->
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <div
                                class="history-item"
                                onclick={() => openTranscript(item.id)}
                                role="button"
                                tabindex="0"
                            >
                                <div class="item-content">
                                    <div class="item-title">
                                        {cleanTitle(item.filename)}
                                    </div>
                                    <div class="item-meta">
                                        <span
                                            >{formatDate(
                                                item.transcribedAt,
                                            )}</span
                                        >
                                        <span>•</span>
                                        <span>{item.speakerCount} speakers</span
                                        >
                                        <span>•</span>
                                        <span
                                            >{item.wordCount.toLocaleString()} words</span
                                        >
                                    </div>
                                    <div class="item-preview">
                                        {item.preview}
                                    </div>
                                </div>
                                <button
                                    class="delete-item-btn"
                                    onclick={(e) => deleteEntry(item.id, e)}
                                    aria-label="Delete transcription"
                                >
                                    🗑️
                                </button>
                            </div>
                        {/each}
                    </div>
                {/if}
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
        max-width: 600px;
        max-height: 80vh;
        display: flex;
        flex-direction: column;
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
    }

    .modal-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
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
        cursor: pointer;
        color: var(--gray-600, #4b5563);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .close-btn:hover {
        background: var(--lavender-dark, #e8e0f0);
        color: var(--navy, #1a2b4a);
    }

    .modal-body {
        flex: 1;
        overflow-y: auto;
        padding: 20px;
    }

    .loading,
    .empty-state {
        text-align: center;
        padding: 40px;
        color: var(--gray-600, #4b5563);
    }

    .spinner {
        width: 32px;
        height: 32px;
        border: 3px solid var(--lavender-dark, #e8e0f0);
        border-top-color: var(--magenta, #e91388);
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin: 0 auto 12px;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .error-message {
        background: var(--error-bg, #fef2f2);
        color: var(--error-color, #ef4444);
        padding: 12px;
        border-radius: 8px;
        margin-bottom: 16px;
    }

    .history-list {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .history-item {
        background: var(--lavender-light, #f8f5fa);
        border: 1px solid var(--lavender-dark, #e8e0f0);
        border-radius: 12px;
        padding: 16px;
        text-align: left;
        cursor: pointer;
        transition: all 0.2s;
        width: 100%;
        display: flex;
        align-items: flex-start;
        gap: 12px;
    }

    .history-item:hover {
        background: var(--lavender, #f0ebf5);
        border-color: var(--magenta, #e91388);
    }

    .item-content {
        flex: 1;
        min-width: 0;
    }

    .item-title {
        font-weight: 600;
        color: var(--navy, #1a2b4a);
        margin-bottom: 4px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .item-meta {
        font-size: 12px;
        color: var(--gray-600, #4b5563);
        margin-bottom: 8px;
        display: flex;
        gap: 8px;
    }

    .item-preview {
        font-size: 13px;
        color: var(--gray-600, #4b5563);
        display: -webkit-box;
        -webkit-line-clamp: 2;
        line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .delete-item-btn {
        background: none;
        border: none;
        cursor: pointer;
        font-size: 14px;
        padding: 4px;
        opacity: 0.5;
        transition: opacity 0.2s;
        flex-shrink: 0;
    }

    .delete-item-btn:hover {
        opacity: 1;
    }
</style>
