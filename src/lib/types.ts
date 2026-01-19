// src/lib/types.ts

// Speaker label modes for identification
export type SpeakerLabelMode =
  | 'generic'         // Keep Speaker A, B, C... (default)
  | 'auto-names'      // Try to auto-detect names from conversation
  | 'known-names'     // User provides known speaker names
  | 'interview'       // Roles: Interviewer, Interviewee
  | 'podcast'         // Roles: Host, Guest
  | 'panel'           // Roles: Moderator, Panelist
  | 'custom-roles';   // User provides custom roles

export const SPEAKER_LABEL_OPTIONS: Record<SpeakerLabelMode, { label: string; type: 'none' | 'name' | 'role'; values?: string[] }> = {
  'generic': { label: 'Generic (Speaker A, B, C...)', type: 'none' },
  'auto-names': { label: 'Auto-detect names', type: 'name', values: [] },
  'known-names': { label: 'Enter known names...', type: 'name' },
  'interview': { label: 'Interview (Interviewer/Interviewee)', type: 'role', values: ['Interviewer', 'Interviewee'] },
  'podcast': { label: 'Podcast (Host/Guest)', type: 'role', values: ['Host', 'Guest'] },
  'panel': { label: 'Panel (Moderator/Panelist)', type: 'role', values: ['Moderator', 'Panelist'] },
  'custom-roles': { label: 'Custom roles...', type: 'role' },
};

// Legacy aliases for backwards compatibility
export type ConversationType = 'none' | 'interview' | 'meeting' | 'panel';
export const CONVERSATION_TYPE_LABELS: Record<ConversationType, string> = {
  'none': 'Generic (Speaker A, B, C...)',
  'interview': 'Interview',
  'meeting': 'Meeting / Presentation',
  'panel': 'Panel Discussion',
};

export interface TranscriptionOptions {
  speakerCount: 'auto' | number;  // 'auto' or 2-20
  speakerLabelMode: SpeakerLabelMode;  // New: how to label speakers
  speakerNamesInput: string;  // Comma-separated names or custom roles
  boostWords: string[];
  boostWordsInput: string;  // Comma-separated string for the new input field
  selectedPresets: string[];  // Array of vocabulary preset IDs
  includeSummary: boolean;
  detectTopics: boolean;
  analyzeSentiment: boolean;
  extractKeyPhrases: boolean;  // New: auto_highlights in AssemblyAI
  // Legacy fields for backwards compatibility
  speakerNames?: string[];
  conversationType?: ConversationType;
}

export interface FileJob {
  id: string;
  filename: string;
  filepath: string;
  status: 'queued' | 'converting' | 'uploading' | 'transcribing' | 'generating' | 'complete' | 'error';
  progress: number;
  error?: string;
  outputPath?: string;
  historyId?: string;  // ID of the saved history entry for navigating to transcript view
}

export interface TranscriptSegment {
  speaker: string;
  text: string;
  start: number;
  end: number;
  sentiment?: 'positive' | 'neutral' | 'negative';
}

export interface TranscriptResult {
  segments: TranscriptSegment[];
  summary?: string;
  topics?: { label: string; relevance: number }[];
}

export const ACCEPTED_EXTENSIONS = [
  '.mp4', '.mov', '.avi', '.mkv', '.webm',
  '.mp3', '.wav', '.m4a', '.aac', '.ogg', '.flac'
];

export const VIDEO_EXTENSIONS = ['.mp4', '.mov', '.avi', '.mkv', '.webm'];
export const AUDIO_EXTENSIONS = ['.mp3', '.wav', '.m4a', '.aac', '.ogg', '.flac'];

/**
 * Extract just the filename from a full path (cross-platform).
 * Handles both Unix paths (/) and Windows paths (\) including UNC paths (\\server\share).
 */
export function extractFilename(path: string): string {
  // Replace all backslashes with forward slashes, then split on forward slash
  const normalized = path.replace(/\\/g, '/');
  const parts = normalized.split('/');
  return parts[parts.length - 1] || 'unknown';
}
