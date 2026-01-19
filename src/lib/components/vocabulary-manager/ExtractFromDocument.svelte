<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { vocabularyStore } from "$lib/stores/vocabulary";

    interface Props {
        isOpen: boolean;
        onClose: () => void;
        openaiApiKey: string;
    }

    let { isOpen, onClose, openaiApiKey }: Props = $props();

    type Step = "upload" | "processing" | "review";

    let currentStep = $state<Step>("upload");
    let selectedFile = $state<{ path: string; name: string } | null>(null);
    let extractedData = $state<ExtractedVocabulary | null>(null);
    let selectedTerms = $state<Set<string>>(new Set());
    let vocabularyName = $state("");
    let error = $state<string | null>(null);
    let processingStatus = $state("");

    interface ExtractedCategory {
        name: string;
        terms: string[];
    }

    interface ExtractedVocabulary {
        categories: ExtractedCategory[];
        suggested_name: string;
    }

    async function handleBrowse() {
        const selected = await open({
            multiple: false,
            filters: [
                {
                    name: "Documents",
                    extensions: ["docx", "pdf", "txt", "md"],
                },
            ],
        });

        if (selected && typeof selected === "string") {
            const name =
                selected.split("/").pop() ||
                selected.split("\\").pop() ||
                "document";
            selectedFile = { path: selected, name };
        }
    }

    async function startExtraction() {
        if (!selectedFile) return;
        if (!openaiApiKey) {
            error = "OpenAI API key not configured. Please add it in Settings.";
            return;
        }

        currentStep = "processing";
        error = null;

        try {
            // Step 1: Extract text from document
            processingStatus = "Reading document...";
            const text = await invoke<string>("extract_document_text", {
                path: selectedFile.path,
            });

            // Step 2: Send to OpenAI for analysis
            processingStatus = "Analyzing with AI...";
            extractedData = await invoke<ExtractedVocabulary>(
                "extract_vocabulary_terms",
                {
                    text,
                    apiKey: openaiApiKey,
                },
            );

            // Initialize all terms as selected
            const allTerms = extractedData.categories.flatMap((c) => c.terms);
            selectedTerms = new Set(allTerms);

            // Use suggested name or derive from filename
            vocabularyName =
                extractedData.suggested_name ||
                selectedFile.name.replace(/\.[^.]+$/, "").replace(/[_-]/g, " ");

            currentStep = "review";
        } catch (e) {
            error = e instanceof Error ? e.message : String(e);
            currentStep = "upload";
        }
    }

    function toggleTerm(term: string) {
        if (selectedTerms.has(term)) {
            selectedTerms.delete(term);
        } else {
            selectedTerms.add(term);
        }
        selectedTerms = new Set(selectedTerms);
    }

    function selectAll() {
        const allTerms =
            extractedData?.categories.flatMap((c) => c.terms) || [];
        selectedTerms = new Set(allTerms);
    }

    function deselectAll() {
        selectedTerms = new Set();
    }

    async function saveVocabulary() {
        if (!vocabularyName.trim() || selectedTerms.size === 0) return;

        try {
            await vocabularyStore.create(
                vocabularyName.trim(),
                "my-vocabularies",
                Array.from(selectedTerms),
            );

            handleClose();
        } catch (e) {
            error = e instanceof Error ? e.message : String(e);
        }
    }

    function handleClose() {
        currentStep = "upload";
        selectedFile = null;
        extractedData = null;
        selectedTerms = new Set();
        vocabularyName = "";
        error = null;
        onClose();
    }

    function getTotalTermCount(): number {
        return (
            extractedData?.categories.reduce(
                (sum, c) => sum + c.terms.length,
                0,
            ) || 0
        );
    }
</script>

{#if isOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-overlay" onclick={handleClose}>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="modal" onclick={(e) => e.stopPropagation()}>
            <div class="panel-header">
                <div class="panel-title">
                    <svg viewBox="0 0 24 24" fill="none" stroke-width="2">
                        <path
                            d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
                        />
                        <polyline points="14 2 14 8 20 8" />
                        <path d="M9 15h6" />
                        <path d="M12 18v-6" />
                    </svg>
                    {#if currentStep === "upload"}
                        Extract Vocabulary from Document
                    {:else if currentStep === "processing"}
                        Analyzing Document
                    {:else}
                        Review Extracted Terms
                    {/if}
                </div>
                <button class="panel-close" onclick={handleClose}>
                    <svg viewBox="0 0 24 24" fill="none" stroke-width="2">
                        <line x1="18" y1="6" x2="6" y2="18" />
                        <line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                </button>
            </div>

            <div class="panel-body">
                {#if error}
                    <div class="error-banner">
                        <svg viewBox="0 0 24 24" fill="none" stroke-width="2">
                            <circle cx="12" cy="12" r="10" />
                            <line x1="12" y1="8" x2="12" y2="12" />
                            <line x1="12" y1="16" x2="12.01" y2="16" />
                        </svg>
                        <span>{error}</span>
                        <button onclick={() => (error = null)}>×</button>
                    </div>
                {/if}

                {#if currentStep === "upload"}
                    <div class="info-banner">
                        <svg viewBox="0 0 24 24" fill="none" stroke-width="2">
                            <circle cx="12" cy="12" r="10" />
                            <path d="M12 16v-4" />
                            <path d="M12 8h.01" />
                        </svg>
                        <p>
                            Upload a document related to your meeting. AI will
                            extract drug names, medical terms, and acronyms to
                            boost transcription accuracy.
                        </p>
                    </div>

                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div
                        class="upload-area"
                        class:has-file={selectedFile}
                        onclick={handleBrowse}
                    >
                        <div class="upload-icon" class:success={selectedFile}>
                            {#if selectedFile}
                                <svg
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke-width="2"
                                >
                                    <polyline points="20 6 9 17 4 12" />
                                </svg>
                            {:else}
                                <svg
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke-width="2"
                                >
                                    <path
                                        d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"
                                    />
                                    <polyline points="17 8 12 3 7 8" />
                                    <line x1="12" y1="3" x2="12" y2="15" />
                                </svg>
                            {/if}
                        </div>
                        <div class="upload-title">
                            {#if selectedFile}
                                {selectedFile.name}
                            {:else}
                                Drop file here or click to browse
                            {/if}
                        </div>
                        <div class="upload-subtitle">
                            {#if selectedFile}
                                Click to choose a different file
                            {:else}
                                Maximum file size: 10MB
                            {/if}
                        </div>
                        {#if !selectedFile}
                            <div class="upload-formats">
                                Supported: .docx, .pdf, .txt
                            </div>
                        {/if}
                    </div>
                {:else if currentStep === "processing"}
                    <div class="processing-state">
                        <div class="processing-spinner"></div>
                        <div class="processing-title">
                            Analyzing document...
                        </div>
                        <div class="processing-status">{processingStatus}</div>
                    </div>
                {:else if currentStep === "review" && extractedData}
                    <div class="review-header">
                        <div>
                            <div class="review-title">Extracted Terms</div>
                            <div class="review-subtitle">
                                {getTotalTermCount()} terms found • Click to toggle
                            </div>
                        </div>
                        <div class="review-actions">
                            <button class="btn-text" onclick={selectAll}
                                >Select All</button
                            >
                            <button class="btn-text" onclick={deselectAll}
                                >Deselect All</button
                            >
                        </div>
                    </div>

                    <div class="terms-scroll">
                        {#each extractedData.categories as category}
                            <div class="term-category">
                                <div class="category-label">
                                    {category.name}
                                    <span class="category-count"
                                        >{category.terms.length}</span
                                    >
                                </div>
                                <div class="term-chips">
                                    {#each category.terms as term}
                                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                                        <span
                                            class="term-chip"
                                            class:selected={selectedTerms.has(
                                                term,
                                            )}
                                            onclick={() => toggleTerm(term)}
                                        >
                                            <span class="term-check">
                                                {#if selectedTerms.has(term)}
                                                    <svg
                                                        viewBox="0 0 24 24"
                                                        fill="none"
                                                        stroke-width="3"
                                                    >
                                                        <polyline
                                                            points="20 6 9 17 4 12"
                                                        />
                                                    </svg>
                                                {/if}
                                            </span>
                                            {term}
                                        </span>
                                    {/each}
                                </div>
                            </div>
                        {/each}
                    </div>

                    <div class="save-form">
                        <label class="form-label">Vocabulary Name</label>
                        <input
                            type="text"
                            class="form-input"
                            bind:value={vocabularyName}
                            placeholder="e.g., Pelacarsen Payer Research"
                        />
                    </div>
                {/if}
            </div>

            <div class="panel-footer">
                {#if currentStep === "upload"}
                    <div></div>
                    <div class="footer-actions">
                        <button class="btn btn-secondary" onclick={handleClose}
                            >Cancel</button
                        >
                        <button
                            class="btn btn-primary"
                            disabled={!selectedFile}
                            onclick={startExtraction}
                        >
                            <svg
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke-width="2"
                                stroke="currentColor"
                            >
                                <circle cx="12" cy="12" r="10" />
                                <path d="M12 6v6l4 2" />
                            </svg>
                            Analyze Document
                        </button>
                    </div>
                {:else if currentStep === "processing"}
                    <div></div>
                    <button class="btn btn-secondary" onclick={handleClose}
                        >Cancel</button
                    >
                {:else}
                    <div class="footer-stats">
                        <strong>{selectedTerms.size}</strong> of {getTotalTermCount()}
                        terms selected
                    </div>
                    <div class="footer-actions">
                        <button
                            class="btn btn-secondary"
                            onclick={() => (currentStep = "upload")}
                            >Back</button
                        >
                        <button
                            class="btn btn-primary"
                            disabled={selectedTerms.size === 0 ||
                                !vocabularyName.trim()}
                            onclick={saveVocabulary}
                        >
                            <svg
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke-width="2"
                                stroke="currentColor"
                            >
                                <path
                                    d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"
                                />
                                <polyline points="17 21 17 13 7 13 7 21" />
                                <polyline points="7 3 7 8 15 8" />
                            </svg>
                            Save Vocabulary
                        </button>
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
        z-index: 1001;
    }

    .modal {
        background: var(--white, #ffffff);
        border-radius: 12px;
        width: 90%;
        max-width: 520px;
        max-height: 90vh;
        display: flex;
        flex-direction: column;
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
    }

    .panel-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 16px 20px;
        border-bottom: 1px solid var(--gray-200, #e5e7eb);
        flex-shrink: 0;
    }

    .panel-title {
        display: flex;
        align-items: center;
        gap: 10px;
        font-size: 16px;
        font-weight: 600;
        color: var(--navy, #1a2b4a);
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
        background: var(--gray-100, #f9fafb);
        border-radius: 8px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
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

    .panel-footer {
        padding: 16px 20px;
        background: var(--gray-100, #f9fafb);
        border-top: 1px solid var(--gray-200, #e5e7eb);
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-shrink: 0;
    }

    .footer-actions {
        display: flex;
        gap: 12px;
    }

    .footer-stats {
        font-size: 12px;
        color: var(--gray-500, #6b7280);
    }

    .footer-stats strong {
        color: var(--magenta, #e91388);
    }

    /* Buttons */
    .btn {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 10px 16px;
        border-radius: 8px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
        border: none;
    }

    .btn svg {
        width: 16px;
        height: 16px;
    }

    .btn-primary {
        background: linear-gradient(
            135deg,
            var(--magenta, #e91388) 0%,
            var(--purple, #6b2d7b) 100%
        );
        color: white;
    }

    .btn-primary:hover:not(:disabled) {
        transform: translateY(-1px);
        box-shadow: 0 4px 12px rgba(233, 19, 136, 0.3);
    }

    .btn-primary:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .btn-secondary {
        background: var(--white, #ffffff);
        color: var(--navy, #1a2b4a);
        border: 1px solid var(--gray-200, #e5e7eb);
    }

    .btn-secondary:hover {
        background: var(--gray-100, #f9fafb);
    }

    .btn-text {
        background: none;
        border: none;
        color: var(--purple, #6b2d7b);
        font-size: 12px;
        cursor: pointer;
        padding: 4px 8px;
        border-radius: 4px;
    }

    .btn-text:hover {
        background: var(--lavender, #f0ebf5);
    }

    /* Banners */
    .info-banner {
        display: flex;
        align-items: flex-start;
        gap: 10px;
        padding: 12px 14px;
        background: var(--lavender-light, #f8f5fa);
        border-radius: 8px;
        margin-bottom: 16px;
    }

    .info-banner svg {
        width: 16px;
        height: 16px;
        stroke: var(--purple, #6b2d7b);
        flex-shrink: 0;
        margin-top: 1px;
    }

    .info-banner p {
        font-size: 12px;
        color: var(--gray-600, #4b5563);
        line-height: 1.5;
        margin: 0;
    }

    .error-banner {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 12px 14px;
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.3);
        border-radius: 8px;
        margin-bottom: 16px;
    }

    .error-banner svg {
        width: 16px;
        height: 16px;
        stroke: var(--error, #ef4444);
        flex-shrink: 0;
    }

    .error-banner span {
        flex: 1;
        font-size: 12px;
        color: var(--error, #ef4444);
    }

    .error-banner button {
        background: none;
        border: none;
        color: var(--error, #ef4444);
        font-size: 18px;
        cursor: pointer;
        padding: 0 4px;
    }

    /* Upload Area */
    .upload-area {
        border: 2px dashed var(--lavender-dark, #e8e0f0);
        border-radius: 12px;
        padding: 32px 20px;
        text-align: center;
        cursor: pointer;
        transition: all 0.2s;
    }

    .upload-area:hover {
        border-color: var(--magenta, #e91388);
        background: rgba(233, 19, 136, 0.02);
    }

    .upload-area.has-file {
        border-style: solid;
        border-color: var(--success, #10b981);
        background: rgba(16, 185, 129, 0.05);
    }

    .upload-icon {
        width: 48px;
        height: 48px;
        background: var(--lavender, #f0ebf5);
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        margin: 0 auto 16px;
    }

    .upload-icon svg {
        width: 24px;
        height: 24px;
        stroke: var(--purple, #6b2d7b);
    }

    .upload-icon.success {
        background: rgba(16, 185, 129, 0.15);
    }

    .upload-icon.success svg {
        stroke: var(--success, #10b981);
    }

    .upload-title {
        font-size: 14px;
        font-weight: 600;
        color: var(--navy, #1a2b4a);
        margin-bottom: 4px;
    }

    .upload-subtitle {
        font-size: 12px;
        color: var(--gray-400, #9ca3af);
    }

    .upload-formats {
        font-size: 11px;
        color: var(--gray-400, #9ca3af);
        margin-top: 12px;
    }

    /* Processing State */
    .processing-state {
        text-align: center;
        padding: 40px 20px;
    }

    .processing-spinner {
        width: 48px;
        height: 48px;
        border: 3px solid var(--lavender, #f0ebf5);
        border-top-color: var(--magenta, #e91388);
        border-radius: 50%;
        margin: 0 auto 20px;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .processing-title {
        font-size: 15px;
        font-weight: 600;
        color: var(--navy, #1a2b4a);
        margin-bottom: 8px;
    }

    .processing-status {
        font-size: 13px;
        color: var(--gray-500, #6b7280);
    }

    /* Review Screen */
    .review-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 16px;
    }

    .review-title {
        font-size: 14px;
        font-weight: 600;
        color: var(--navy, #1a2b4a);
    }

    .review-subtitle {
        font-size: 12px;
        color: var(--gray-400, #9ca3af);
        margin-top: 2px;
    }

    .review-actions {
        display: flex;
        gap: 4px;
    }

    .terms-scroll {
        max-height: 280px;
        overflow-y: auto;
        padding-right: 8px;
    }

    .term-category {
        margin-bottom: 16px;
    }

    .term-category:last-child {
        margin-bottom: 0;
    }

    .category-label {
        font-size: 11px;
        font-weight: 600;
        color: var(--gray-500, #6b7280);
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin-bottom: 8px;
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .category-count {
        font-size: 10px;
        font-weight: 400;
        background: var(--gray-100, #f9fafb);
        padding: 2px 6px;
        border-radius: 4px;
    }

    .term-chips {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
    }

    .term-chip {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 6px 10px;
        background: var(--lavender-light, #f8f5fa);
        border: 1px solid var(--lavender-dark, #e8e0f0);
        border-radius: 6px;
        font-size: 12px;
        color: var(--navy, #1a2b4a);
        cursor: pointer;
        transition: all 0.15s;
    }

    .term-chip:hover {
        background: var(--lavender, #f0ebf5);
    }

    .term-chip.selected {
        background: linear-gradient(
            135deg,
            rgba(233, 19, 136, 0.1) 0%,
            rgba(107, 45, 123, 0.1) 100%
        );
        border-color: var(--magenta, #e91388);
        color: var(--magenta, #e91388);
    }

    .term-check {
        width: 14px;
        height: 14px;
        border: 1.5px solid var(--gray-300, #d1d5db);
        border-radius: 3px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .term-chip.selected .term-check {
        background: var(--magenta, #e91388);
        border-color: var(--magenta, #e91388);
    }

    .term-check svg {
        width: 10px;
        height: 10px;
        stroke: white;
    }

    /* Save Form */
    .save-form {
        margin-top: 16px;
        padding-top: 16px;
        border-top: 1px solid var(--gray-200, #e5e7eb);
    }

    .form-label {
        font-size: 12px;
        font-weight: 500;
        color: var(--navy, #1a2b4a);
        margin-bottom: 6px;
        display: block;
    }

    .form-input {
        width: 100%;
        padding: 10px 12px;
        border: 1px solid var(--gray-200, #e5e7eb);
        border-radius: 8px;
        font-size: 13px;
        color: var(--navy, #1a2b4a);
    }

    .form-input:focus {
        outline: none;
        border-color: var(--magenta, #e91388);
        box-shadow: 0 0 0 3px rgba(233, 19, 136, 0.1);
    }
</style>
