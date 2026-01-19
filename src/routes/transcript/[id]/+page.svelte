<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/stores";
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import {
        getHistoryEntry,
        deleteHistoryEntry,
        updateHistoryEntry,
        type HistoryEntry,
    } from "$lib/services/history";
    import { getApiKey } from "$lib/services/transcription";
    import {
        generateWordDocument,
        saveDocument,
    } from "$lib/services/docx-export";
    import { save } from "@tauri-apps/plugin-dialog";

    let entry: HistoryEntry | null = $state(null);
    let loading = $state(true);
    let error = $state<string | null>(null);
    let exporting = $state(false);
    let identifying = $state(false);
    let speakerMapping = $state<Record<string, string>>({});

    // Audio player state
    let audioElement: HTMLAudioElement | null = $state(null);
    let isPlaying = $state(false);
    let currentTime = $state(0);
    let duration = $state(0);
    let currentSegmentIndex = $state(-1);

    // Convert audio path to asset URL for Tauri WebView
    const audioSrc = $derived(
        entry?.audioPath ? convertFileSrc(entry.audioPath) : null,
    );

    const id = $derived($page.params.id);

    onMount(async () => {
        await loadEntry();
    });

    async function loadEntry() {
        if (!id) {
            error = "No transcript ID provided";
            loading = false;
            return;
        }
        loading = true;
        error = null;
        try {
            entry = await getHistoryEntry(id);
        } catch (e) {
            error = `Failed to load transcript: ${e instanceof Error ? e.message : String(e)}`;
            console.error(error);
        } finally {
            loading = false;
        }
    }

    async function exportToWord() {
        if (!entry) return;

        exporting = true;
        error = null;
        try {
            const docBuffer = await generateWordDocument(entry.transcript, {
                filename: entry.filename,
                transcribedDate: new Date(entry.transcribedAt),
                includeSummary: entry.options.includedSummary,
                includeTopics: entry.options.includedTopics,
                includeSentiment: entry.options.includedSentiment,
            });

            const defaultName =
                entry.filename.replace(/\.[^/.]+$/, "") + "_transcript.docx";
            const outputPath = await save({
                defaultPath: defaultName,
                filters: [{ name: "Word Document", extensions: ["docx"] }],
                title: "Save Transcript As",
            });

            if (!outputPath) {
                exporting = false;
                return;
            }

            await saveDocument(docBuffer, outputPath);
            alert(`Document saved to: ${outputPath}`);
        } catch (e) {
            error = `Failed to export: ${e instanceof Error ? e.message : String(e)}`;
            console.error(error);
        } finally {
            exporting = false;
        }
    }

    async function handleDelete() {
        if (!entry) return;
        if (
            !confirm(
                "Are you sure you want to delete this transcription from history?",
            )
        )
            return;

        try {
            await deleteHistoryEntry(entry.id);
            goto("/");
        } catch (e) {
            error = `Failed to delete: ${e instanceof Error ? e.message : String(e)}`;
        }
    }

    async function identifySpeakers() {
        if (!entry) return;

        identifying = true;
        error = null;

        try {
            // Get API key
            const apiKey = await getApiKey();
            if (!apiKey) {
                error =
                    "AssemblyAI API key not configured. Please add it in Settings.";
                identifying = false;
                return;
            }

            // Build transcript text with speaker labels
            const transcriptText = entry.transcript.segments
                .map((s) => `Speaker ${s.speaker}:\n${s.text}`)
                .join("\n\n");

            // Get unique speaker labels
            const uniqueSpeakers = [
                ...new Set(entry.transcript.segments.map((s) => s.speaker)),
            ];

            // Call LeMUR to identify speakers
            const mapping = await invoke<Record<string, string>>(
                "identify_speakers",
                {
                    transcriptText,
                    speakerLabels: uniqueSpeakers,
                    apiKey,
                },
            );

            speakerMapping = mapping;

            // Apply mapping to transcript segments
            if (Object.keys(mapping).length > 0) {
                const updatedSegments = entry.transcript.segments.map(
                    (segment) => ({
                        ...segment,
                        speaker: mapping[segment.speaker] || segment.speaker,
                    }),
                );

                entry = {
                    ...entry,
                    transcript: {
                        ...entry.transcript,
                        segments: updatedSegments,
                    },
                };

                // Save updated entry to history
                await updateHistoryEntry(entry);
            }
        } catch (e) {
            error = `Failed to identify speakers: ${e instanceof Error ? e.message : String(e)}`;
            console.error(error);
        } finally {
            identifying = false;
        }
    }

    function formatDate(isoString: string): string {
        const date = new Date(isoString);
        return date.toLocaleDateString("en-US", {
            weekday: "long",
            month: "long",
            day: "numeric",
            year: "numeric",
            hour: "numeric",
            minute: "2-digit",
        });
    }

    function formatTime(ms: number): string {
        const minutes = Math.floor(ms / 60000);
        const seconds = Math.floor((ms % 60000) / 1000);
        return `${minutes}:${String(seconds).padStart(2, "0")}`;
    }

    function goBack() {
        goto("/");
    }

    // Clean up filename for display title
    function cleanTitle(filename: string): string {
        return filename
            .replace(/\.[^/.]+$/, "") // Remove extension
            .replace(/_/g, " ") // Underscores to spaces
            .replace(/\s+/g, " ") // Normalize spaces
            .trim();
    }

    // Audio player functions
    function handleTimeUpdate() {
        if (audioElement) {
            currentTime = audioElement.currentTime * 1000; // Convert to ms
            updateCurrentSegment();
        }
    }

    function handleLoadedMetadata() {
        if (audioElement) {
            duration = audioElement.duration * 1000; // Convert to ms
        }
    }

    function updateCurrentSegment() {
        if (!entry) return;

        // Find segment that contains current time
        const index = entry.transcript.segments.findIndex((seg, i, arr) => {
            const nextStart = arr[i + 1]?.start ?? Infinity;
            return currentTime >= seg.start && currentTime < nextStart;
        });

        if (index !== currentSegmentIndex) {
            currentSegmentIndex = index;

            // Auto-scroll to current segment
            if (index >= 0) {
                const el = document.getElementById(`segment-${index}`);
                el?.scrollIntoView({ behavior: "smooth", block: "center" });
            }
        }
    }

    function seekToSegment(segmentIndex: number) {
        if (!entry || !audioElement) return;

        const segment = entry.transcript.segments[segmentIndex];
        if (segment) {
            audioElement.currentTime = segment.start / 1000; // Convert ms to seconds
            audioElement.play();
            isPlaying = true;
        }
    }

    function togglePlayPause() {
        if (!audioElement) return;

        if (isPlaying) {
            audioElement.pause();
        } else {
            audioElement.play();
        }
        isPlaying = !isPlaying;
    }

    function formatPlayTime(ms: number): string {
        const totalSeconds = Math.floor(ms / 1000);
        const minutes = Math.floor(totalSeconds / 60);
        const seconds = totalSeconds % 60;
        return `${minutes}:${seconds.toString().padStart(2, "0")}`;
    }
</script>

<svelte:head>
    <title
        >{entry ? cleanTitle(entry.filename) : "Transcript"} | OHG Scribe</title
    >
</svelte:head>

<main class="transcript-page">
    <header class="header">
        <button class="back-btn" onclick={goBack}>
            <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
            >
                <path d="M19 12H5M12 19l-7-7 7-7" />
            </svg>
            Back
        </button>

        {#if entry}
            <div class="header-actions">
                <button
                    class="identify-btn"
                    onclick={identifySpeakers}
                    disabled={identifying}
                    title="Use AI to identify speaker names from transcript"
                >
                    {identifying ? "Identifying..." : "🪄 Identify Speakers"}
                </button>
                <button
                    class="export-btn"
                    onclick={exportToWord}
                    disabled={exporting}
                >
                    {exporting ? "Exporting..." : "Export to Word"}
                </button>
                <button class="delete-btn" onclick={handleDelete}>
                    Delete
                </button>
            </div>
        {/if}
    </header>

    <div class="content">
        {#if loading}
            <div class="loading">
                <div class="spinner"></div>
                <p>Loading transcript...</p>
            </div>
        {:else if error}
            <div class="error-state">
                <p class="error-message">{error}</p>
                <button class="retry-btn" onclick={loadEntry}>Try Again</button>
            </div>
        {:else if entry}
            <div class="transcript-header">
                <h1>{cleanTitle(entry.filename)}</h1>
                <div class="meta-row">
                    <span class="meta-item"
                        >📅 {formatDate(entry.transcribedAt)}</span
                    >
                    <span class="meta-item"
                        >👥 {entry.speakerCount} speakers</span
                    >
                    <span class="meta-item"
                        >📝 {entry.wordCount.toLocaleString()} words</span
                    >
                </div>
            </div>

            <!-- Audio Player -->
            {#if audioSrc}
                <div class="audio-player">
                    <audio
                        bind:this={audioElement}
                        src={audioSrc}
                        ontimeupdate={handleTimeUpdate}
                        onloadedmetadata={handleLoadedMetadata}
                        onplay={() => (isPlaying = true)}
                        onpause={() => (isPlaying = false)}
                        onended={() => {
                            isPlaying = false;
                            currentSegmentIndex = -1;
                        }}
                    />
                    <button class="play-btn" onclick={togglePlayPause}>
                        {isPlaying ? "⏸️" : "▶️"}
                    </button>
                    <div class="time-display">
                        {formatPlayTime(currentTime)} / {formatPlayTime(
                            duration,
                        )}
                    </div>
                    <input
                        type="range"
                        class="progress-slider"
                        min="0"
                        max={duration}
                        value={currentTime}
                        oninput={(e) => {
                            if (audioElement) {
                                audioElement.currentTime =
                                    Number(e.currentTarget.value) / 1000;
                            }
                        }}
                    />
                </div>
            {/if}

            <div class="transcript-body">
                {#each entry.transcript.segments as segment, i}
                    <div
                        id={`segment-${i}`}
                        class="segment"
                        class:even={i % 2 === 1}
                        class:playing={i === currentSegmentIndex}
                        onclick={() => seekToSegment(i)}
                        role="button"
                        tabindex="0"
                    >
                        <div class="segment-header">
                            <span class="speaker">{segment.speaker}</span>
                            <span class="timestamp"
                                >{formatTime(segment.start)}</span
                            >
                        </div>
                        <p class="segment-text">{segment.text}</p>
                    </div>
                {/each}
            </div>
        {:else}
            <div class="empty-state">
                <p>Transcript not found.</p>
                <button class="back-btn-large" onclick={goBack}>Go Back</button>
            </div>
        {/if}
    </div>
</main>

<style>
    .transcript-page {
        height: 100vh;
        overflow: hidden;
        background: linear-gradient(
            180deg,
            var(--lavender-light, #f8f5fa) 0%,
            var(--white, #ffffff) 100%
        );
        display: flex;
        flex-direction: column;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px 24px;
        background: rgba(255, 255, 255, 1);
        border-bottom: 1px solid var(--lavender-dark, #e8e0f0);
        position: sticky;
        top: 0;
        z-index: 100;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
    }

    .back-btn {
        display: flex;
        align-items: center;
        gap: 8px;
        background: none;
        border: none;
        color: var(--magenta, #e91388);
        font-size: 14px;
        font-weight: 500;
        cursor: pointer;
        padding: 8px 12px;
        border-radius: 8px;
        transition: all 0.2s;
    }

    .back-btn:hover {
        background: var(--lavender, #f0ebf5);
    }

    .header-actions {
        display: flex;
        gap: 12px;
    }

    .identify-btn {
        background: var(--white, #ffffff);
        color: var(--purple, #6b2d7b);
        border: 1px solid var(--purple, #6b2d7b);
        padding: 10px 20px;
        border-radius: 8px;
        font-size: 14px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
    }

    .identify-btn:hover:not(:disabled) {
        background: var(--lavender-light, #f8f5fa);
        transform: translateY(-1px);
        box-shadow: 0 4px 12px rgba(107, 45, 123, 0.2);
    }

    .identify-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .export-btn {
        background: linear-gradient(
            135deg,
            var(--magenta, #e91388) 0%,
            var(--purple, #6b2d7b) 100%
        );
        color: white;
        border: none;
        padding: 10px 20px;
        border-radius: 8px;
        font-size: 14px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
    }

    .export-btn:hover:not(:disabled) {
        transform: translateY(-1px);
        box-shadow: 0 4px 12px rgba(233, 19, 136, 0.3);
    }

    .export-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .delete-btn {
        background: var(--white, #ffffff);
        color: var(--error-color, #ef4444);
        border: 1px solid var(--error-color, #ef4444);
        padding: 10px 20px;
        border-radius: 8px;
        font-size: 14px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
    }

    .delete-btn:hover {
        background: var(--error-bg, #fef2f2);
    }

    .content {
        flex: 1;
        max-width: 900px;
        width: 100%;
        margin: 0 auto;
        padding: 32px 24px;
        overflow-y: auto;
    }

    .loading,
    .empty-state,
    .error-state {
        text-align: center;
        padding: 60px 20px;
        color: var(--gray-600, #4b5563);
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 3px solid var(--lavender-dark, #e8e0f0);
        border-top-color: var(--magenta, #e91388);
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin: 0 auto 16px;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .error-message {
        background: var(--error-bg, #fef2f2);
        color: var(--error-color, #ef4444);
        padding: 16px;
        border-radius: 8px;
        margin-bottom: 16px;
    }

    .retry-btn,
    .back-btn-large {
        background: var(--lavender, #f0ebf5);
        border: none;
        color: var(--navy, #1a2b4a);
        padding: 12px 24px;
        border-radius: 8px;
        font-size: 14px;
        cursor: pointer;
    }

    .transcript-header {
        margin-bottom: 32px;
    }

    .transcript-header h1 {
        margin: 0 0 12px;
        font-size: 24px;
        font-weight: 600;
        color: var(--navy, #1a2b4a);
        word-break: break-word;
    }

    .meta-row {
        display: flex;
        flex-wrap: wrap;
        gap: 20px;
    }

    .meta-item {
        font-size: 14px;
        color: var(--gray-600, #4b5563);
    }

    .transcript-body {
        background: var(--white, #ffffff);
        border-radius: 16px;
        padding: 24px;
        padding-bottom: 100px; /* Space for fixed audio player */
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
    }

    .segment {
        padding: 16px 0;
        border-bottom: 1px solid var(--lavender-dark, #e8e0f0);
    }

    .segment:last-child {
        border-bottom: none;
    }

    .segment-header {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-bottom: 8px;
    }

    .speaker {
        font-weight: 600;
        color: var(--magenta, #e91388);
    }

    .segment.even .speaker {
        color: var(--purple, #6b2d7b);
    }

    .timestamp {
        font-size: 12px;
        color: var(--gray-400, #9ca3af);
        background: var(--lavender-light, #f8f5fa);
        padding: 2px 8px;
        border-radius: 4px;
    }

    .segment-text {
        margin: 0;
        font-size: 15px;
        line-height: 1.7;
        color: var(--navy, #1a2b4a);
    }

    /* Audio player - fixed to bottom */
    .audio-player {
        display: flex;
        align-items: center;
        gap: 12px;
        background: var(--white, #ffffff);
        border-radius: 12px 12px 0 0;
        padding: 16px 20px;
        box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.1);
        position: fixed;
        bottom: 0;
        left: 0;
        right: 0;
        z-index: 100;
    }

    .play-btn {
        background: linear-gradient(
            135deg,
            var(--magenta, #e91388) 0%,
            var(--purple, #6b2d7b) 100%
        );
        border: none;
        border-radius: 50%;
        width: 48px;
        height: 48px;
        font-size: 20px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s;
        flex-shrink: 0;
    }

    .play-btn:hover {
        transform: scale(1.05);
        box-shadow: 0 4px 12px rgba(233, 19, 136, 0.3);
    }

    .time-display {
        font-size: 13px;
        color: var(--gray-600, #4b5563);
        font-variant-numeric: tabular-nums;
        min-width: 85px;
    }

    .progress-slider {
        flex: 1;
        height: 6px;
        -webkit-appearance: none;
        appearance: none;
        background: var(--lavender-dark, #e8e0f0);
        border-radius: 3px;
        cursor: pointer;
    }

    .progress-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 14px;
        height: 14px;
        border-radius: 50%;
        background: var(--magenta, #e91388);
        cursor: pointer;
    }

    /* Clickable and playing segments */
    .segment {
        padding: 16px;
        border-bottom: 1px solid var(--lavender-dark, #e8e0f0);
        cursor: pointer;
        border-radius: 8px;
        margin: 0 -16px;
        transition:
            background 0.2s,
            border-color 0.2s;
    }

    .segment:hover {
        background: var(--lavender-light, #f8f5fa);
    }

    .segment.playing {
        background: linear-gradient(
            90deg,
            rgba(233, 19, 136, 0.08) 0%,
            rgba(107, 45, 123, 0.08) 100%
        );
        border-left: 3px solid var(--magenta, #e91388);
        padding-left: 13px;
    }
</style>
