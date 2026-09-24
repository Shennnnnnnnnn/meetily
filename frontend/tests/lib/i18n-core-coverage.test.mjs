import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const frontendRoot = join(dirname(fileURLToPath(import.meta.url)), '..', '..');

const forbiddenByFile = {
  'src/components/TranscriptView.tsx': [
    'Welcome to meetily!',
    'Start recording to see live transcription',
    'Listening...',
  ],
  'src/components/EmptyStateSummary.tsx': [
    'No Summary Generated Yet',
    'Generate an AI-powered summary of your meeting transcript to get key points, action items, and decisions.',
    'Generate Summary',
  ],
  'src/components/MeetingDetails/TranscriptButtonGroup.tsx': [
    '>Copy<',
    '>Recording<',
    '>Enhance<',
    'Copy Transcript',
    'Open Recording Folder',
  ],
  'src/components/MeetingDetails/SummaryGeneratorButtonGroup.tsx': [
    '>Generate Summary<',
    '>Regenerate Summary<',
    '>AI Model<',
    '>Template<',
  ],
  'src/components/WhisperModelManager.tsx': [
    'This may take a few minutes',
    'Model downloaded and ready to use',
    'The download is still shutting down.',
  ],
  'src/components/ParakeetModelManager.tsx': [
    'This may take a few minutes',
    'Model downloaded and ready to use',
    'The download is still shutting down.',
  ],
};

test('core meeting UI does not render known hard-coded English strings', () => {
  const failures = [];

  for (const [relativePath, forbiddenStrings] of Object.entries(forbiddenByFile)) {
    const source = readFileSync(join(frontendRoot, relativePath), 'utf8');
    for (const value of forbiddenStrings) {
      if (source.includes(value)) failures.push(`${relativePath}: ${value}`);
    }
  }

  assert.deepEqual(failures, []);
});
