// src/lib/services/docx-export.ts
import {
    Document,
    Packer,
    Paragraph,
    TextRun,
    AlignmentType,
    BorderStyle,
    convertInchesToTwip
} from 'docx';
import type { TranscriptResult, TranscriptSegment } from '../types';

// Speaker colors - rotate through these for different speakers
const SPEAKER_COLORS = [
    '2563eb', // blue
    'dc2626', // red
    '16a34a', // green
    '9333ea', // purple
    'ea580c', // orange
    '0891b2', // cyan
];

// Sentiment emojis
const SENTIMENT_EMOJI: Record<string, string> = {
    positive: '😊',
    neutral: '😐',
    negative: '😟'
};

// Clean up filename for display title
function cleanTitle(filename: string): string {
    return filename
        .replace(/\.[^/.]+$/, '')     // Remove extension
        .replace(/_/g, ' ')           // Underscores to spaces
        .replace(/\s+/g, ' ')         // Normalize spaces
        .trim();
}

// Get unique speakers
function getUniqueSpeakers(segments: TranscriptSegment[]): string[] {
    const speakers = new Set<string>();
    segments.forEach(s => speakers.add(s.speaker));
    return Array.from(speakers);
}

// Calculate word count
function getWordCount(segments: TranscriptSegment[]): number {
    return segments.reduce((count, segment) => {
        return count + segment.text.split(/\s+/).filter(w => w).length;
    }, 0);
}

// Get total duration
function getTotalDuration(segments: TranscriptSegment[]): number {
    if (segments.length === 0) return 0;
    return segments[segments.length - 1].end;
}

// Format duration for display
function formatDuration(ms: number): string {
    const totalSeconds = Math.floor(ms / 1000);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);

    if (hours > 0) return `${hours}h ${minutes}m`;
    if (minutes > 0) return `${minutes}m`;
    return '< 1m';
}

// Format milliseconds to MM:SS or H:MM:SS (no brackets, no leading zeros for hours)
function formatTimestamp(ms: number): string {
    const totalSeconds = Math.floor(ms / 1000);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;

    if (hours > 0) {
        return `${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

// Generate a horizontal rule paragraph
function createHorizontalRule(): Paragraph {
    return new Paragraph({
        border: {
            bottom: {
                color: 'cccccc',
                space: 1,
                style: BorderStyle.SINGLE,
                size: 6,
            },
        },
        spacing: {
            before: 200,
            after: 200,
        },
    });
}

// Create section header
function createSectionHeader(title: string): Paragraph {
    return new Paragraph({
        children: [
            new TextRun({
                text: title,
                bold: true,
                size: 24, // 12pt
                allCaps: true,
                color: '6b7280',
            }),
        ],
        spacing: {
            before: 400,
            after: 200,
        },
    });
}

export interface ExportOptions {
    filename: string;
    transcribedDate: Date;
    includeSummary: boolean;
    includeTopics: boolean;
    includeSentiment: boolean;
}

export async function generateWordDocument(
    result: TranscriptResult,
    options: ExportOptions
): Promise<Uint8Array> {
    const children: Paragraph[] = [];

    // Calculate metadata upfront
    const duration = getTotalDuration(result.segments);
    const speakers = getUniqueSpeakers(result.segments);
    const wordCount = getWordCount(result.segments);

    // Assign speaker colors
    const speakerColorMap: Record<string, string> = {};
    let colorIndex = 0;
    speakers.forEach(speaker => {
        speakerColorMap[speaker] = SPEAKER_COLORS[colorIndex % SPEAKER_COLORS.length];
        colorIndex++;
    });

    // ========== HEADER ==========

    // Title (cleaned)
    children.push(
        new Paragraph({
            children: [
                new TextRun({
                    text: cleanTitle(options.filename),
                    bold: true,
                    size: 36, // 18pt
                    color: '1a2b4a',
                }),
            ],
            spacing: { after: 80 },
        })
    );

    // Metadata line: duration • speakers • words • date
    children.push(
        new Paragraph({
            children: [
                new TextRun({
                    text: formatDuration(duration),
                    size: 20,
                    color: '6b7280',
                }),
                new TextRun({
                    text: '  •  ',
                    size: 20,
                    color: 'cccccc',
                }),
                new TextRun({
                    text: `${speakers.length} speaker${speakers.length !== 1 ? 's' : ''}`,
                    size: 20,
                    color: '6b7280',
                }),
                new TextRun({
                    text: '  •  ',
                    size: 20,
                    color: 'cccccc',
                }),
                new TextRun({
                    text: `${wordCount.toLocaleString()} words`,
                    size: 20,
                    color: '6b7280',
                }),
                new TextRun({
                    text: '  •  ',
                    size: 20,
                    color: 'cccccc',
                }),
                new TextRun({
                    text: `Transcribed ${options.transcribedDate.toLocaleDateString('en-US', {
                        year: 'numeric',
                        month: 'short',
                        day: 'numeric'
                    })}`,
                    size: 20,
                    color: '6b7280',
                }),
            ],
            spacing: { after: 200 },
        })
    );

    // Participants line
    const participantRuns: TextRun[] = [
        new TextRun({
            text: 'Participants: ',
            size: 20,
            color: '9ca3af',
        }),
    ];

    speakers.forEach((speaker, i) => {
        participantRuns.push(
            new TextRun({
                text: speaker,
                size: 20,
                color: speakerColorMap[speaker],
                bold: true,
            })
        );
        if (i < speakers.length - 1) {
            participantRuns.push(new TextRun({
                text: ', ',
                size: 20,
                color: '9ca3af',
            }));
        }
    });

    children.push(
        new Paragraph({
            children: participantRuns,
            spacing: { after: 300 },
        })
    );

    children.push(createHorizontalRule());

    // Summary section (if enabled and available)
    if (options.includeSummary && result.summary) {
        children.push(createSectionHeader('Summary'));

        // Parse bullets from summary
        const bulletPoints = result.summary.split('\n').filter(line => line.trim());
        for (const point of bulletPoints) {
            const cleanPoint = point.replace(/^[•\-]\s*/, '');
            children.push(
                new Paragraph({
                    children: [
                        new TextRun({
                            text: `• ${cleanPoint}`,
                            size: 22,
                            color: '374151',
                        }),
                    ],
                    spacing: { before: 100, after: 100 },
                    indent: { left: convertInchesToTwip(0.25) },
                })
            );
        }
        children.push(createHorizontalRule());
    }

    // Topics section (if enabled and available)
    if (options.includeTopics && result.topics && result.topics.length > 0) {
        children.push(createSectionHeader('Topics Detected'));

        // Show top 5 topics
        const topTopics = result.topics.slice(0, 5);
        for (const topic of topTopics) {
            children.push(
                new Paragraph({
                    children: [
                        new TextRun({
                            text: `• ${topic.label} `,
                            size: 22,
                            color: '374151',
                        }),
                        new TextRun({
                            text: `(${Math.round(topic.relevance)}%)`,
                            color: '9ca3af',
                            size: 20,
                        }),
                    ],
                    spacing: { before: 80, after: 80 },
                    indent: { left: convertInchesToTwip(0.25) },
                })
            );
        }
        children.push(createHorizontalRule());
    }

    // ========== TRANSCRIPT ==========
    children.push(createSectionHeader('Transcript'));

    for (const segment of result.segments) {
        const speakerColor = speakerColorMap[segment.speaker];

        // Speaker line: "Speaker Name  0:00"
        const speakerRuns: TextRun[] = [
            new TextRun({
                text: segment.speaker,
                bold: true,
                color: speakerColor,
                size: 22,
            }),
            new TextRun({
                text: `  ${formatTimestamp(segment.start)}`,
                color: '9ca3af',
                size: 18, // Smaller timestamp
            }),
        ];

        // Add sentiment if enabled
        if (options.includeSentiment && segment.sentiment) {
            speakerRuns.push(new TextRun({
                text: `  ${SENTIMENT_EMOJI[segment.sentiment] || ''}`,
                size: 22,
            }));
        }

        children.push(
            new Paragraph({
                children: speakerRuns,
                spacing: { before: 240, after: 60 },
            })
        );

        // Transcript text
        children.push(
            new Paragraph({
                children: [
                    new TextRun({
                        text: segment.text,
                        size: 22,
                        color: '374151',
                    }),
                ],
                spacing: { after: 120 },
                alignment: AlignmentType.LEFT,
            })
        );
    }

    // Create document
    const doc = new Document({
        sections: [
            {
                properties: {
                    page: {
                        margin: {
                            top: convertInchesToTwip(1),
                            right: convertInchesToTwip(1),
                            bottom: convertInchesToTwip(1),
                            left: convertInchesToTwip(1),
                        },
                    },
                },
                children,
            },
        ],
        styles: {
            default: {
                document: {
                    run: {
                        font: 'Calibri',
                        size: 22, // 11pt
                    },
                    paragraph: {
                        spacing: {
                            line: 276, // 1.15 line spacing
                        },
                    },
                },
            },
        },
    });

    // Generate the document as a Blob (works in browser/Tauri context)
    try {
        console.log('Starting Packer.toBlob...');
        const blob = await Packer.toBlob(doc);
        console.log('Blob created, size:', blob.size);

        // Convert Blob to Uint8Array
        const arrayBuffer = await blob.arrayBuffer();
        console.log('ArrayBuffer created, size:', arrayBuffer.byteLength);
        return new Uint8Array(arrayBuffer);
    } catch (error) {
        console.error('Error generating Word document:', error);
        throw new Error(`Failed to generate Word document: ${error instanceof Error ? error.message : String(error)}`);
    }
}

// Save document to file, with auto-increment if file exists
export async function saveDocument(
    buffer: Uint8Array,
    outputPath: string
): Promise<string> {  // Now returns the actual path used
    try {
        console.log('Importing @tauri-apps/plugin-fs...');
        const { writeFile, exists } = await import('@tauri-apps/plugin-fs');

        // Check if file exists and find a unique name
        let finalPath = outputPath;
        let counter = 1;

        while (await exists(finalPath)) {
            // Extract base path and extension
            const lastDotIndex = outputPath.lastIndexOf('.');
            const basePath = lastDotIndex > 0 ? outputPath.slice(0, lastDotIndex) : outputPath;
            const extension = lastDotIndex > 0 ? outputPath.slice(lastDotIndex) : '';

            // Remove any existing counter suffix (e.g., "_1", "_2")
            const baseWithoutCounter = basePath.replace(/_\d+$/, '');

            finalPath = `${baseWithoutCounter}_${counter}${extension}`;
            counter++;

            // Safety limit to prevent infinite loop
            if (counter > 100) {
                throw new Error('Too many files with the same name');
            }
        }

        console.log('Calling writeFile with path:', finalPath, 'buffer size:', buffer.length);
        await writeFile(finalPath, buffer);
        console.log('File written successfully!');

        return finalPath;
    } catch (error) {
        console.error('Error saving document:', error);
        throw new Error(`Failed to save document: ${error instanceof Error ? error.message : String(error)}`);
    }
}

