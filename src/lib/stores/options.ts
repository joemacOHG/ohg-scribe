// src/lib/stores/options.ts
import { writable } from 'svelte/store';
import type { TranscriptionOptions } from '../types';

const defaultOptions: TranscriptionOptions = {
    speakerCount: 'auto',
    speakerLabelMode: 'generic',
    speakerNamesInput: '',
    boostWords: [],
    boostWordsInput: '',
    selectedPresets: [],
    includeSummary: false,
    detectTopics: false,
    analyzeSentiment: false,
    extractKeyPhrases: false,
};

function createOptionsStore() {
    const { subscribe, update, set } = writable<TranscriptionOptions>(defaultOptions);

    return {
        subscribe,
        update: (updates: Partial<TranscriptionOptions>) => {
            update(options => ({ ...options, ...updates }));
        },
        setSpeakerNames: (namesString: string) => {
            update(options => ({
                ...options,
                speakerNamesInput: namesString,
                speakerNames: namesString
                    .split(',')
                    .map(name => name.trim())
                    .filter(name => name.length > 0)
            }));
        },
        // Set boost words from comma-separated string (legacy)
        setBoostWords: (wordsString: string) => {
            update(options => ({
                ...options,
                boostWordsInput: wordsString,
                boostWords: wordsString
                    .split(',')
                    .map(word => word.trim())
                    .filter(word => word.length > 0)
            }));
        },
        // Set boost words from array (new tag interface)
        setBoostWordsArray: (words: string[]) => {
            update(options => ({
                ...options,
                boostWords: words
            }));
        },
        // Set just the input value (for typing in progress)
        setBoostWordsInput: (input: string) => {
            update(options => ({
                ...options,
                boostWordsInput: input
            }));
        },
        setSelectedPresets: (presets: string[]) => {
            update(options => ({
                ...options,
                selectedPresets: presets
            }));
        },
        reset: () => set(defaultOptions)
    };
}

export const optionsStore = createOptionsStore();

