// src/lib/services/transcription.ts
import { invoke } from '@tauri-apps/api/core';
import type { TranscriptionOptions, TranscriptResult, TranscriptSegment } from '../types';

export interface ConversionResult {
    output_path: string;
    temp_dir: string;
}

export interface RustTranscriptionOptions {
    max_speakers: number | null;  // null = auto (1-10), number = max speakers
    boost_words: string[];
    include_summary: boolean;
    detect_topics: boolean;
    analyze_sentiment: boolean;
    extract_key_phrases: boolean;  // auto_highlights in AssemblyAI
    speaker_label_mode: string;    // 'generic' | 'auto-names' | 'known-names' | 'interview' | etc.
    speaker_values: string[];      // Names or custom roles from user input
}

export interface TranscriptResponse {
    id: string;
    status: string;
    text?: string;
    utterances?: Utterance[];
    summary?: string;
    iab_categories_result?: {
        summary: Record<string, number>;
    };
    sentiment_analysis_results?: SentimentResult[];
    auto_highlights_result?: {
        results: { text: string; count: number; rank: number }[];
    };
    error?: string;
}

export interface Utterance {
    speaker: string;
    text: string;
    start: number;
    end: number;
}

export interface SentimentResult {
    text: string;
    start: number;
    end: number;
    sentiment: 'POSITIVE' | 'NEGATIVE' | 'NEUTRAL';
    confidence: number;
    speaker?: string;
}

// Settings commands
export async function getApiKey(): Promise<string | null> {
    return await invoke<string | null>('get_api_key');
}

export async function setApiKey(apiKey: string): Promise<void> {
    return await invoke('set_api_key', { apiKey });
}

export async function deleteApiKey(): Promise<void> {
    return await invoke('delete_api_key');
}

// OpenAI key commands
export async function getOpenAIKey(): Promise<string | null> {
    return await invoke<string | null>('get_openai_key');
}

export async function setOpenAIKey(apiKey: string): Promise<void> {
    return await invoke('set_openai_key', { apiKey });
}

// FFmpeg conversion
export async function convertToAudio(inputPath: string): Promise<ConversionResult> {
    return await invoke<ConversionResult>('convert_to_audio', { inputPath });
}

export async function cleanupTempDir(tempDir: string): Promise<void> {
    return await invoke('cleanup_temp_dir', { tempDir });
}

// AssemblyAI API
export async function uploadAudio(filePath: string, apiKey: string): Promise<string> {
    return await invoke<string>('upload_audio', { filePath, apiKey });
}

export async function submitTranscription(
    uploadUrl: string,
    apiKey: string,
    options: TranscriptionOptions
): Promise<string> {
    // Parse speaker values from comma-separated input
    const speakerValues = options.speakerNamesInput
        ? options.speakerNamesInput.split(',').map(s => s.trim()).filter(s => s.length > 0)
        : [];

    const rustOptions: RustTranscriptionOptions = {
        max_speakers: options.speakerCount === 'auto' ? null : options.speakerCount,
        boost_words: options.boostWords,
        include_summary: options.includeSummary,
        detect_topics: options.detectTopics,
        analyze_sentiment: options.analyzeSentiment,
        extract_key_phrases: options.extractKeyPhrases,
        speaker_label_mode: options.speakerLabelMode,
        speaker_values: speakerValues
    };
    return await invoke<string>('submit_transcription', { uploadUrl, apiKey, options: rustOptions });
}

export async function pollTranscription(
    transcriptId: string,
    apiKey: string
): Promise<TranscriptResponse> {
    return await invoke<TranscriptResponse>('poll_transcription', { transcriptId, apiKey });
}

// Poll until transcription is complete
export async function waitForTranscription(
    transcriptId: string,
    apiKey: string,
    onProgress?: (status: string) => void,
    initialDelayMs: number = 5000,
    pollIntervalMs: number = 3000,
    timeoutMs: number = 30 * 60 * 1000 // 30 minutes
): Promise<TranscriptResponse> {
    const startTime = Date.now();

    // Initial delay
    await new Promise(resolve => setTimeout(resolve, initialDelayMs));

    while (Date.now() - startTime < timeoutMs) {
        const response = await pollTranscription(transcriptId, apiKey);

        onProgress?.(response.status);

        if (response.status === 'completed') {
            return response;
        }

        if (response.status === 'error') {
            throw new Error(response.error || 'Transcription failed');
        }

        // Wait before next poll
        await new Promise(resolve => setTimeout(resolve, pollIntervalMs));
    }

    throw new Error('Transcription timed out');
}

// Convert raw API response to our segment format
export function parseTranscriptResponse(
    response: TranscriptResponse,
    speakerNames: string[]
): TranscriptResult {
    const segments: TranscriptSegment[] = [];

    if (response.utterances) {
        // Map speaker labels to names
        const speakerMap: Record<string, string> = {};
        let speakerIndex = 0;

        for (const utterance of response.utterances) {
            if (!speakerMap[utterance.speaker]) {
                if (speakerIndex < speakerNames.length && speakerNames[speakerIndex]) {
                    speakerMap[utterance.speaker] = speakerNames[speakerIndex];
                } else {
                    speakerMap[utterance.speaker] = utterance.speaker;
                }
                speakerIndex++;
            }

            // Find sentiment for this segment if available
            let sentiment: 'positive' | 'neutral' | 'negative' | undefined;
            if (response.sentiment_analysis_results) {
                const sentimentResult = response.sentiment_analysis_results.find(
                    s => s.start >= utterance.start && s.end <= utterance.end
                );
                if (sentimentResult) {
                    sentiment = sentimentResult.sentiment.toLowerCase() as 'positive' | 'neutral' | 'negative';
                }
            }

            segments.push({
                speaker: speakerMap[utterance.speaker],
                text: utterance.text,
                start: utterance.start,
                end: utterance.end,
                sentiment
            });
        }
    }

    // Parse topics
    const topics: { label: string; relevance: number }[] = [];
    if (response.iab_categories_result?.summary) {
        for (const [label, relevance] of Object.entries(response.iab_categories_result.summary)) {
            topics.push({ label, relevance: relevance * 100 });
        }
        // Sort by relevance
        topics.sort((a, b) => b.relevance - a.relevance);
    }

    return {
        segments,
        summary: response.summary,
        topics: topics.length > 0 ? topics : undefined
    };
}
