<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import DropZone from "$lib/components/DropZone.svelte";
  import FileQueue from "$lib/components/FileQueue.svelte";
  import OptionsPanel from "$lib/components/OptionsPanel.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import HistoryPanel from "$lib/components/HistoryPanel.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import { queueStore } from "$lib/stores/queue";
  import { optionsStore } from "$lib/stores/options";
  import type { FileJob, TranscriptionOptions } from "$lib/types";
  import {
    getApiKey,
    setApiKey as saveApiKey,
    getOpenAIKey,
    setOpenAIKey as saveOpenAIKey,
    convertToAudio,
    cleanupTempDir,
    uploadAudio,
    submitTranscription,
    waitForTranscription,
    parseTranscriptResponse,
  } from "$lib/services/transcription";
  import {
    generateWordDocument,
    saveDocument,
  } from "$lib/services/docx-export";
  import { saveToHistory, createHistoryEntry } from "$lib/services/history";
  import { openPath } from "@tauri-apps/plugin-opener";

  let settingsOpen = $state(false);
  let historyOpen = $state(false);
  let apiKey = $state("");
  let openaiKey = $state("");
  let jobs: FileJob[] = $state([]);
  let options: TranscriptionOptions = $state({
    speakerCount: "auto",
    speakerLabelMode: "generic",
    speakerNamesInput: "",
    boostWords: [],
    boostWordsInput: "",
    selectedPresets: [],
    includeSummary: false,
    detectTopics: false,
    analyzeSentiment: false,
    extractKeyPhrases: false,
  });
  let toasts: {
    id: string;
    message: string;
    type: "success" | "error" | "info";
  }[] = $state([]);
  let isProcessing = $state(false);

  queueStore.subscribe((value) => {
    jobs = value;
  });

  optionsStore.subscribe((value) => {
    options = value;
  });

  onMount(async () => {
    // Load API keys from storage
    try {
      const storedKey = await getApiKey();
      if (storedKey) {
        apiKey = storedKey;
      }
      const storedOpenAIKey = await getOpenAIKey();
      if (storedOpenAIKey) {
        openaiKey = storedOpenAIKey;
      }
    } catch (e) {
      console.error("Failed to load API keys:", e);
    }
  });

  function handleFilesDropped(files: { filename: string; filepath: string }[]) {
    queueStore.addFiles(files);
    showToast(
      `Added ${files.length} file${files.length > 1 ? "s" : ""} to queue`,
      "info",
    );

    // Start processing if not already processing
    if (!isProcessing) {
      processQueue();
    }
  }

  async function processQueue() {
    if (isProcessing) return;

    // Check for API key
    if (!apiKey) {
      showToast("Please add your AssemblyAI API key in Settings", "error");
      settingsOpen = true;
      return;
    }

    isProcessing = true;

    // Process files one at a time
    while (true) {
      const queuedJob = jobs.find((j) => j.status === "queued");
      if (!queuedJob) break;

      await processFile(queuedJob.id);
    }

    isProcessing = false;
  }

  async function processFile(jobId: string) {
    const job = jobs.find((j) => j.id === jobId);
    if (!job) return;

    let tempDir: string | undefined;

    try {
      // Step 1: Convert to audio
      queueStore.updateJob(jobId, { status: "converting", progress: 10 });

      const conversionResult = await convertToAudio(job.filepath);
      tempDir = conversionResult.temp_dir;
      const audioPath = conversionResult.output_path;

      queueStore.updateJob(jobId, { progress: 25 });

      // Step 2: Upload to AssemblyAI
      queueStore.updateJob(jobId, { status: "uploading", progress: 30 });

      const uploadUrl = await uploadAudio(audioPath, apiKey);

      queueStore.updateJob(jobId, { progress: 45 });

      // Step 3: Submit transcription
      queueStore.updateJob(jobId, { status: "transcribing", progress: 50 });

      const transcriptId = await submitTranscription(
        uploadUrl,
        apiKey,
        options,
      );

      // Step 4: Wait for completion
      const response = await waitForTranscription(
        transcriptId,
        apiKey,
        (status) => {
          // Update progress based on status
          if (status === "processing") {
            queueStore.updateJob(jobId, { progress: 65 });
          }
        },
      );

      queueStore.updateJob(jobId, { progress: 80 });

      // Step 5: Parse transcript and generate Word doc
      queueStore.updateJob(jobId, { status: "generating", progress: 85 });

      console.log("Parsing transcript response...");
      console.log(
        "Response utterances count:",
        response.utterances?.length || 0,
      );

      const speakerNames = options.speakerNamesInput
        ? options.speakerNamesInput
            .split(",")
            .map((s) => s.trim())
            .filter((s) => s.length > 0)
        : [];

      const transcriptResult = parseTranscriptResponse(response, speakerNames);

      console.log(
        "Parsed transcript segments:",
        transcriptResult.segments.length,
      );

      console.log("Generating Word document...");
      const docBuffer = await generateWordDocument(transcriptResult, {
        filename: job.filename,
        transcribedDate: new Date(),
        includeSummary: options.includeSummary,
        includeTopics: options.detectTopics,
        includeSentiment: options.analyzeSentiment,
      });

      console.log("Word document generated, buffer size:", docBuffer.length);

      // Step 6: Save document (auto-increments if file exists)
      const requestedPath = job.filepath.replace(
        /\.[^/.]+$/,
        "_transcript.docx",
      );
      console.log("Saving document to:", requestedPath);

      const actualPath = await saveDocument(docBuffer, requestedPath);
      console.log("Document saved successfully to:", actualPath);

      queueStore.updateJob(jobId, {
        status: "complete",
        progress: 100,
        outputPath: actualPath,
      });

      // Save to history
      try {
        const historyEntry = createHistoryEntry(
          job.filename,
          job.filepath,
          transcriptResult,
          {
            speakerNames: speakerNames,
            includedSummary: options.includeSummary,
            includedTopics: options.detectTopics,
            includedSentiment: options.analyzeSentiment,
          },
        );

        // Store audio file for playback in transcript view
        try {
          const storedAudioPath = await invoke<string>("store_audio_file", {
            sourcePath: audioPath,
            historyId: historyEntry.id,
          });
          historyEntry.audioPath = storedAudioPath;
          console.log("Audio stored at:", storedAudioPath);
        } catch (audioError) {
          console.warn("Failed to store audio for playback:", audioError);
        }

        await saveToHistory(historyEntry);
        console.log("Saved to history:", historyEntry.id);

        // Store historyId in job so View button can navigate
        queueStore.updateJob(jobId, { historyId: historyEntry.id });
      } catch (historyError) {
        console.warn("Failed to save to history:", historyError);
      }

      showToast(`Completed: ${job.filename}`, "success");
    } catch (error) {
      console.error("Error processing file:", error);
      queueStore.updateJob(jobId, {
        status: "error",
        error: error instanceof Error ? error.message : "Unknown error",
      });
      showToast(
        `Error: ${error instanceof Error ? error.message : "Failed to process file"}`,
        "error",
      );
    } finally {
      // Clean up temp directory
      if (tempDir) {
        try {
          await cleanupTempDir(tempDir);
        } catch (e) {
          console.warn("Failed to cleanup temp dir:", e);
        }
      }
    }
  }

  async function handleOpenFile(path: string) {
    console.log("Attempting to open file:", path);
    try {
      await openPath(path);
      console.log("File opened successfully");
    } catch (e) {
      console.error("Failed to open file:", path, "Error:", e);
      showToast(
        `Failed to open file: ${e instanceof Error ? e.message : String(e)}`,
        "error",
      );
    }
  }

  function handleRetry(id: string) {
    queueStore.updateJob(id, {
      status: "queued",
      progress: 0,
      error: undefined,
    });
    if (!isProcessing) {
      processQueue();
    }
  }

  async function handleSaveApiKeys(
    assemblyaiKeyVal: string,
    openaiKeyVal: string,
  ) {
    try {
      await saveApiKey(assemblyaiKeyVal);
      apiKey = assemblyaiKeyVal;

      if (openaiKeyVal) {
        await saveOpenAIKey(openaiKeyVal);
      }
      openaiKey = openaiKeyVal;

      showToast("Settings saved", "success");

      // Start processing if there are queued files
      if (!isProcessing && jobs.some((j) => j.status === "queued")) {
        processQueue();
      }
    } catch (e) {
      console.error("Failed to save API keys:", e);
      showToast("Failed to save settings", "error");
    }
  }

  function showToast(
    message: string,
    type: "success" | "error" | "info" = "info",
  ) {
    const id = Date.now().toString();
    toasts = [...toasts, { id, message, type }];
  }

  function removeToast(id: string) {
    toasts = toasts.filter((t) => t.id !== id);
  }

  function handleViewTranscript(job: FileJob) {
    if (job.historyId) {
      goto(`/transcript/${job.historyId}`);
    } else {
      showToast("Transcript not available", "error");
    }
  }

  let hasJobs = $derived(jobs.length > 0);
</script>

<main class="app">
  <header class="header">
    <div class="logo">
      <div class="logo-icon">
        <svg viewBox="0 0 32 32" fill="none">
          <circle
            cx="16"
            cy="16"
            r="10"
            fill="none"
            stroke="white"
            stroke-width="1.5"
          />
          <path
            d="M11 16 V16 M13 13 V19 M16 10 V22 M19 13 V19 M21 15 V17"
            stroke="white"
            stroke-width="1.5"
            stroke-linecap="round"
          />
        </svg>
      </div>
      <h1 class="logo-text">
        <span class="logo-ohg">OHG</span><span class="logo-scribe">Scribe</span>
      </h1>
    </div>
    <div class="header-actions">
      <button
        class="header-btn"
        onclick={() => (historyOpen = true)}
        aria-label="History"
        title="Transcription History"
      >
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </button>
      <button
        class="header-btn"
        onclick={() => (settingsOpen = true)}
        aria-label="Settings"
        title="Settings"
      >
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path
            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
          />
          <circle cx="12" cy="12" r="3" />
        </svg>
      </button>
    </div>
  </header>

  <div class="content">
    <DropZone onFilesDropped={handleFilesDropped} compact={hasJobs} />

    {#if hasJobs}
      <FileQueue
        {jobs}
        onOpen={handleOpenFile}
        onRetry={handleRetry}
        onViewTranscript={handleViewTranscript}
      />
    {/if}

    <OptionsPanel openaiApiKey={openaiKey} />
  </div>

  <SettingsModal
    isOpen={settingsOpen}
    onClose={() => (settingsOpen = false)}
    onSave={handleSaveApiKeys}
    currentAssemblyAIKey={apiKey}
    currentOpenAIKey={openaiKey}
  />

  <HistoryPanel isOpen={historyOpen} onClose={() => (historyOpen = false)} />

  {#each toasts as toast (toast.id)}
    <Toast
      message={toast.message}
      type={toast.type}
      onClose={() => removeToast(toast.id)}
    />
  {/each}
</main>

<style>
  :global(:root) {
    /* Primary */
    --magenta: #e91388;
    --magenta-light: #f74ba5;
    --magenta-dark: #c4106f;

    /* Secondary */
    --purple: #6b2d7b;
    --purple-light: #8b4d9b;

    /* Neutral */
    --navy: #1a2b4a;
    --navy-light: #2d4066;

    /* Backgrounds */
    --lavender-light: #f8f5fa;
    --lavender: #f0ebf5;
    --lavender-dark: #e8e0f0;

    /* Grays */
    --gray-100: #f9fafb;
    --gray-200: #e5e7eb;
    --gray-400: #9ca3af;
    --gray-600: #4b5563;

    /* Semantic */
    --success-color: #10b981;
    --error-color: #ef4444;
    --error-bg: #fef2f2;
    --white: #ffffff;

    /* Legacy mappings for compatibility */
    --accent-color: var(--magenta);
    --accent-hover: var(--magenta-dark);
    --accent-light: var(--lavender);
    --text-primary: var(--navy);
    --text-secondary: var(--gray-600);
    --bg-primary: var(--white);
    --bg-secondary: var(--lavender-light);
    --bg-hover: var(--lavender);
    --border-color: var(--lavender-dark);
  }

  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
      sans-serif;
    background: linear-gradient(
      180deg,
      var(--lavender-light) 0%,
      var(--white) 100%
    );
    color: var(--navy);
    -webkit-font-smoothing: antialiased;
    min-height: 100vh;
  }

  .app {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 24px;
    border-bottom: 1px solid var(--lavender-dark);
    background: var(--white);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .logo-icon {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    background: linear-gradient(135deg, var(--magenta) 0%, var(--purple) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .logo-icon svg {
    width: 24px;
    height: 24px;
  }

  .logo-text {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    display: flex;
    gap: 0;
  }

  .logo-ohg {
    color: var(--magenta);
  }

  .logo-scribe {
    color: var(--navy);
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .header-btn {
    width: 36px;
    height: 36px;
    background: var(--white);
    border: none;
    cursor: pointer;
    padding: 8px;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--gray-600);
  }

  .header-btn:hover {
    background: var(--lavender);
    color: var(--navy);
  }

  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 24px;
    max-width: 600px;
    width: 100%;
    margin: 0 auto;
  }
</style>
