<script lang="ts">
  import { optionsStore } from "$lib/stores/options";
  import type { TranscriptionOptions, SpeakerLabelMode } from "$lib/types";
  import SpeakersSection from "./options/SpeakersSection.svelte";
  import VocabularySection from "./options/VocabularySection.svelte";
  import AnalysisSection from "./options/AnalysisSection.svelte";
  import VocabularyManager from "./vocabulary-manager/VocabularyManager.svelte";
  import ExtractFromDocument from "./vocabulary-manager/ExtractFromDocument.svelte";

  interface Props {
    openaiApiKey?: string;
  }

  let { openaiApiKey = "" }: Props = $props();

  let isExpanded = $state(false);
  let vocabManagerOpen = $state(false);
  let extractModalOpen = $state(false);

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

  optionsStore.subscribe((value) => {
    options = value;
  });

  // Speakers handlers
  function handleSpeakerCountChange(value: "auto" | number) {
    optionsStore.update({ speakerCount: value });
  }

  function handleSpeakerLabelModeChange(value: SpeakerLabelMode) {
    optionsStore.update({ speakerLabelMode: value });
    // Clear the names input when switching modes
    if (value !== "known-names" && value !== "custom-roles") {
      optionsStore.update({ speakerNamesInput: "" });
    }
  }

  function handleSpeakerNamesInputChange(value: string) {
    optionsStore.update({ speakerNamesInput: value });
  }

  // Vocabulary handlers
  function handleBoostWordsChange(words: string[]) {
    optionsStore.setBoostWordsArray(words);
  }

  function handleBoostWordsInputChange(value: string) {
    optionsStore.setBoostWordsInput(value);
  }

  function handlePresetsChange(presets: string[]) {
    optionsStore.setSelectedPresets(presets);
  }

  function handleOpenManager() {
    vocabManagerOpen = true;
  }

  function handleCloseManager() {
    vocabManagerOpen = false;
  }

  function handleOpenExtract() {
    extractModalOpen = true;
  }

  function handleCloseExtract() {
    extractModalOpen = false;
  }

  // Analysis handlers
  function handleSummaryChange(value: boolean) {
    optionsStore.update({ includeSummary: value });
  }

  function handleTopicsChange(value: boolean) {
    optionsStore.update({ detectTopics: value });
  }

  function handleSentimentChange(value: boolean) {
    optionsStore.update({ analyzeSentiment: value });
  }

  function handleKeyPhrasesChange(value: boolean) {
    optionsStore.update({ extractKeyPhrases: value });
  }
</script>

<div class="options-panel">
  <button class="toggle-btn" onclick={() => (isExpanded = !isExpanded)}>
    <span class="toggle-icon">{isExpanded ? "▲" : "▼"}</span>
    Customize
  </button>

  {#if isExpanded}
    <div class="options-content">
      <SpeakersSection
        speakerCount={options.speakerCount}
        speakerLabelMode={options.speakerLabelMode}
        speakerNamesInput={options.speakerNamesInput}
        onSpeakerCountChange={handleSpeakerCountChange}
        onSpeakerLabelModeChange={handleSpeakerLabelModeChange}
        onSpeakerNamesInputChange={handleSpeakerNamesInputChange}
      />

      <VocabularySection
        boostWords={options.boostWords}
        boostWordsInput={options.boostWordsInput}
        selectedPresets={options.selectedPresets}
        onBoostWordsChange={handleBoostWordsChange}
        onBoostWordsInputChange={handleBoostWordsInputChange}
        onPresetsChange={handlePresetsChange}
        onOpenManager={handleOpenManager}
        onOpenExtract={handleOpenExtract}
      />

      <AnalysisSection
        bind:includeSummary={options.includeSummary}
        bind:detectTopics={options.detectTopics}
        bind:analyzeSentiment={options.analyzeSentiment}
        bind:extractKeyPhrases={options.extractKeyPhrases}
        onSummaryChange={handleSummaryChange}
        onTopicsChange={handleTopicsChange}
        onSentimentChange={handleSentimentChange}
        onKeyPhrasesChange={handleKeyPhrasesChange}
      />
    </div>
  {/if}
</div>

<!-- Vocabulary Manager Modal -->
<VocabularyManager
  isOpen={vocabManagerOpen}
  onClose={handleCloseManager}
  onOpenExtract={handleOpenExtract}
/>

<!-- Extract from Document Modal -->
<ExtractFromDocument
  isOpen={extractModalOpen}
  onClose={handleCloseExtract}
  {openaiApiKey}
/>

<style>
  .options-panel {
    background: var(--white, #ffffff);
    border-radius: 12px;
    overflow: hidden;
    margin-bottom: 16px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  }

  .toggle-btn {
    width: 100%;
    padding: 14px 20px;
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    border: none;
    font-size: 14px;
    font-weight: 500;
    color: var(--navy, #1a2b4a);
    cursor: pointer;
    transition: background 0.2s;
  }

  .toggle-btn:hover {
    background: var(--lavender-light, #f8f5fa);
  }

  .toggle-icon {
    font-size: 10px;
    color: var(--gray-400, #9ca3af);
  }

  .options-content {
    background: var(--white, #ffffff);
  }
</style>
