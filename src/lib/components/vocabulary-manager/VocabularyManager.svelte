<script lang="ts">
    import { vocabularyStore } from "$lib/stores/vocabulary";
    import { onMount } from "svelte";
    import type { Vocabulary, VocabularyCategory } from "$lib/types/vocabulary";

    interface Props {
        isOpen: boolean;
        onClose: () => void;
        onOpenExtract?: () => void;
    }

    let { isOpen, onClose, onOpenExtract }: Props = $props();

    let searchQuery = $state("");
    let expandedCategories = $state<Set<string>>(new Set());

    onMount(() => {
        vocabularyStore.load();
    });

    function toggleCategory(categoryId: string) {
        if (expandedCategories.has(categoryId)) {
            expandedCategories.delete(categoryId);
            expandedCategories = new Set(expandedCategories);
        } else {
            expandedCategories.add(categoryId);
            expandedCategories = new Set(expandedCategories);
        }
    }

    function getFilteredVocabs(): Map<VocabularyCategory, Vocabulary[]> {
        const byCategory = vocabularyStore.getByCategory();
        if (!searchQuery) return byCategory;

        const lowerQuery = searchQuery.toLowerCase();
        const result = new Map<VocabularyCategory, Vocabulary[]>();

        for (const [category, vocabs] of byCategory) {
            const filtered = vocabs.filter((v) =>
                v.name.toLowerCase().includes(lowerQuery),
            );
            if (filtered.length > 0) {
                result.set(category, filtered);
            }
        }

        return result;
    }

    async function handleDuplicate(vocab: Vocabulary) {
        const newName = `${vocab.name} (Copy)`;
        try {
            await vocabularyStore.duplicate(vocab.id, newName);
        } catch (e) {
            console.error("Failed to duplicate vocabulary:", e);
        }
    }

    async function handleDelete(vocab: Vocabulary) {
        if (vocab.isSystem) return;
        try {
            await vocabularyStore.delete(vocab.id);
        } catch (e) {
            console.error("Failed to delete vocabulary:", e);
        }
    }

    $effect(() => {
        // Re-render when store updates
        $vocabularyStore;
    });
</script>

{#if isOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-overlay" onclick={onClose}>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="modal" onclick={(e) => e.stopPropagation()}>
            <div class="panel-header">
                <div class="panel-title">
                    <svg viewBox="0 0 24 24" fill="none" stroke-width="2">
                        <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
                        <path
                            d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"
                        />
                    </svg>
                    Vocabulary Manager
                </div>
                <button class="panel-close" onclick={onClose}>
                    <svg viewBox="0 0 24 24" fill="none" stroke-width="2">
                        <line x1="18" y1="6" x2="6" y2="18" />
                        <line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                </button>
            </div>

            <div class="panel-body">
                <div class="vocab-list-header">
                    <div class="search-box">
                        <svg viewBox="0 0 24 24" fill="none" stroke-width="2">
                            <circle cx="11" cy="11" r="8" />
                            <line x1="21" y1="21" x2="16.65" y2="16.65" />
                        </svg>
                        <input
                            type="text"
                            placeholder="Search vocabularies..."
                            bind:value={searchQuery}
                        />
                    </div>
                </div>

                <!-- Action Buttons -->
                <div class="action-buttons">
                    <button
                        class="btn btn-primary"
                        onclick={() => {
                            /* TODO: New Vocab */
                        }}
                    >
                        <svg
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke-width="2"
                            stroke="currentColor"
                        >
                            <line x1="12" y1="5" x2="12" y2="19" />
                            <line x1="5" y1="12" x2="19" y2="12" />
                        </svg>
                        New Vocabulary
                    </button>
                    <button class="btn btn-outline" onclick={onOpenExtract}>
                        <svg
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke-width="2"
                            stroke="currentColor"
                        >
                            <path
                                d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
                            />
                            <polyline points="14 2 14 8 20 8" />
                            <path d="M9 15h6" />
                            <path d="M12 18v-6" />
                        </svg>
                        Extract from Document
                    </button>
                </div>

                <div class="categories-list">
                    {#if $vocabularyStore.loading}
                        <div class="loading">Loading vocabularies...</div>
                    {:else}
                        {#each [...getFilteredVocabs()] as [category, vocabs]}
                            <div
                                class="category"
                                class:open={expandedCategories.has(category.id)}
                            >
                                <div
                                    class="category-header"
                                    onclick={() => toggleCategory(category.id)}
                                >
                                    <div class="category-left">
                                        <svg
                                            class="category-chevron"
                                            viewBox="0 0 24 24"
                                            fill="none"
                                            stroke-width="2"
                                        >
                                            <polyline points="9 18 15 12 9 6" />
                                        </svg>
                                        <span class="category-name"
                                            >{category.name}</span
                                        >
                                        <span class="category-count"
                                            >{vocabs.length} vocabularies</span
                                        >
                                    </div>
                                    <span
                                        class="category-badge"
                                        class:system={category.isSystem}
                                        class:user={!category.isSystem}
                                    >
                                        {category.isSystem
                                            ? "System"
                                            : "Custom"}
                                    </span>
                                </div>

                                {#if expandedCategories.has(category.id)}
                                    <div class="category-items">
                                        {#each vocabs as vocab}
                                            <div class="vocab-item">
                                                <div class="vocab-item-left">
                                                    <div class="vocab-icon">
                                                        <svg
                                                            viewBox="0 0 24 24"
                                                            fill="none"
                                                            stroke-width="2"
                                                        >
                                                            <path
                                                                d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"
                                                            />
                                                        </svg>
                                                    </div>
                                                    <div>
                                                        <div class="vocab-name">
                                                            {vocab.name}
                                                        </div>
                                                        <div class="vocab-meta">
                                                            {vocab.terms.length}
                                                            terms • {vocab.isSystem
                                                                ? "System preset"
                                                                : "Custom"}
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class="vocab-item-actions">
                                                    <button
                                                        class="btn btn-ghost"
                                                        title="Duplicate"
                                                        onclick={() =>
                                                            handleDuplicate(
                                                                vocab,
                                                            )}
                                                    >
                                                        <svg
                                                            viewBox="0 0 24 24"
                                                            fill="none"
                                                            stroke-width="2"
                                                            stroke="currentColor"
                                                        >
                                                            <rect
                                                                x="9"
                                                                y="9"
                                                                width="13"
                                                                height="13"
                                                                rx="2"
                                                                ry="2"
                                                            />
                                                            <path
                                                                d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
                                                            />
                                                        </svg>
                                                    </button>
                                                    {#if !vocab.isSystem}
                                                        <button
                                                            class="btn btn-ghost btn-danger"
                                                            title="Delete"
                                                            onclick={() =>
                                                                handleDelete(
                                                                    vocab,
                                                                )}
                                                        >
                                                            <svg
                                                                viewBox="0 0 24 24"
                                                                fill="none"
                                                                stroke-width="2"
                                                                stroke="currentColor"
                                                            >
                                                                <polyline
                                                                    points="3 6 5 6 21 6"
                                                                />
                                                                <path
                                                                    d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
                                                                />
                                                            </svg>
                                                        </button>
                                                    {/if}
                                                </div>
                                            </div>
                                        {/each}
                                    </div>
                                {/if}
                            </div>
                        {/each}
                    {/if}
                </div>
            </div>

            <div class="panel-footer">
                <button class="btn btn-secondary" onclick={onClose}
                    >Close</button
                >
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
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }

    .modal {
        background: var(--white, #ffffff);
        border-radius: 12px;
        width: 500px;
        max-height: 80vh;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    }

    .panel-header {
        padding: 16px 20px;
        background: var(--lavender-light, #f8f5fa);
        border-bottom: 1px solid var(--lavender-dark, #e8e0f0);
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .panel-title {
        font-size: 16px;
        font-weight: 600;
        color: var(--navy, #1a2b4a);
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .panel-title svg {
        width: 20px;
        height: 20px;
        stroke: var(--purple, #6b2d7b);
    }

    .panel-close {
        width: 32px;
        height: 32px;
        border: none;
        background: var(--white, #ffffff);
        border-radius: 8px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background 0.2s;
    }

    .panel-close:hover {
        background: var(--lavender, #f0ebf5);
    }

    .panel-close svg {
        width: 16px;
        height: 16px;
        stroke: var(--gray-500, #6b7280);
    }

    .panel-body {
        padding: 20px;
        overflow-y: auto;
        flex: 1;
    }

    .vocab-list-header {
        margin-bottom: 16px;
    }

    .search-box {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        background: var(--gray-100, #f9fafb);
        border: 1px solid var(--gray-200, #e5e7eb);
        border-radius: 8px;
    }

    .search-box svg {
        width: 16px;
        height: 16px;
        stroke: var(--gray-400, #9ca3af);
    }

    .search-box input {
        border: none;
        background: transparent;
        font-size: 13px;
        color: var(--navy, #1a2b4a);
        width: 100%;
        outline: none;
    }

    .loading {
        text-align: center;
        color: var(--gray-400, #9ca3af);
        padding: 40px;
    }

    .categories-list {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .category {
        border: 1px solid var(--gray-200, #e5e7eb);
        border-radius: 10px;
        overflow: hidden;
    }

    .category-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 12px 16px;
        background: var(--gray-100, #f9fafb);
        cursor: pointer;
        user-select: none;
    }

    .category-header:hover {
        background: var(--lavender-light, #f8f5fa);
    }

    .category-left {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .category-chevron {
        width: 18px;
        height: 18px;
        stroke: var(--gray-400, #9ca3af);
        transition: transform 0.2s;
    }

    .category.open .category-chevron {
        transform: rotate(90deg);
    }

    .category-name {
        font-size: 14px;
        font-weight: 600;
        color: var(--navy, #1a2b4a);
    }

    .category-count {
        font-size: 12px;
        color: var(--gray-400, #9ca3af);
        background: var(--white, #ffffff);
        padding: 2px 8px;
        border-radius: 10px;
    }

    .category-badge {
        font-size: 10px;
        padding: 2px 8px;
        border-radius: 4px;
        font-weight: 500;
    }

    .category-badge.system {
        background: var(--lavender, #f0ebf5);
        color: var(--purple, #6b2d7b);
    }

    .category-badge.user {
        background: rgba(233, 19, 136, 0.1);
        color: var(--magenta, #e91388);
    }

    .category-items {
        border-top: 1px solid var(--gray-200, #e5e7eb);
    }

    .vocab-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 12px 16px 12px 44px;
        border-bottom: 1px solid var(--gray-100, #f9fafb);
        transition: background 0.15s;
    }

    .vocab-item:last-child {
        border-bottom: none;
    }

    .vocab-item:hover {
        background: var(--lavender-light, #f8f5fa);
    }

    .vocab-item-left {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .vocab-icon {
        width: 32px;
        height: 32px;
        background: var(--lavender, #f0ebf5);
        border-radius: 6px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .vocab-icon svg {
        width: 16px;
        height: 16px;
        stroke: var(--purple, #6b2d7b);
    }

    .vocab-name {
        font-size: 14px;
        color: var(--navy, #1a2b4a);
        font-weight: 500;
    }

    .vocab-meta {
        font-size: 12px;
        color: var(--gray-400, #9ca3af);
        margin-top: 2px;
    }

    .vocab-item-actions {
        display: flex;
        gap: 4px;
        opacity: 0;
        transition: opacity 0.15s;
    }

    .vocab-item:hover .vocab-item-actions {
        opacity: 1;
    }

    .btn {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 8px 16px;
        border-radius: 8px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
        border: none;
    }

    .btn svg {
        width: 14px;
        height: 14px;
    }

    .btn-secondary {
        background: var(--white, #ffffff);
        color: var(--navy, #1a2b4a);
        border: 1px solid var(--gray-200, #e5e7eb);
    }

    .btn-secondary:hover {
        background: var(--gray-100, #f9fafb);
    }

    .btn-ghost {
        background: transparent;
        color: var(--gray-500, #6b7280);
        padding: 6px;
    }

    .btn-ghost:hover {
        background: var(--gray-100, #f9fafb);
        color: var(--navy, #1a2b4a);
    }

    .btn-danger {
        color: var(--error, #ef4444);
    }

    .btn-danger:hover {
        background: rgba(239, 68, 68, 0.1);
    }

    .panel-footer {
        padding: 16px 20px;
        background: var(--gray-100, #f9fafb);
        border-top: 1px solid var(--gray-200, #e5e7eb);
        display: flex;
        justify-content: flex-end;
        gap: 12px;
    }

    /* Action Buttons */
    .action-buttons {
        display: flex;
        gap: 8px;
        margin-bottom: 16px;
    }

    .btn-primary {
        background: linear-gradient(
            135deg,
            var(--magenta, #e91388) 0%,
            var(--purple, #6b2d7b) 100%
        );
        color: white;
    }

    .btn-primary:hover {
        transform: translateY(-1px);
        box-shadow: 0 4px 12px rgba(233, 19, 136, 0.3);
    }

    .btn-outline {
        background: transparent;
        color: var(--purple, #6b2d7b);
        border: 1px solid var(--lavender-dark, #e8e0f0);
    }

    .btn-outline:hover {
        background: var(--lavender-light, #f8f5fa);
    }
</style>
