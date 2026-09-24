type Translate = (key: string, values?: Record<string, string | number>) => string;

type ProgressScope = 'import' | 'retranscribe';

const STAGE_KEYS: Record<ProgressScope, Record<string, string>> = {
  import: {
    copying: 'import.stageCopying',
    decoding: 'import.stageDecoding',
    resampling: 'import.stageResampling',
    vad: 'import.stageVad',
    transcribing: 'import.stageTranscribing',
    saving: 'import.stageSaving',
    complete: 'import.stageComplete',
  },
  retranscribe: {
    decoding: 'retranscribe.stageDecoding',
    resampling: 'retranscribe.stageResampling',
    vad: 'retranscribe.stageVad',
    transcribing: 'retranscribe.stageTranscribing',
    saving: 'retranscribe.stageSaving',
    complete: 'retranscribe.stageComplete',
  },
};

export function localizeProgressStage(t: Translate, scope: ProgressScope, stage: string): string {
  const key = STAGE_KEYS[scope][stage];
  return key ? t(key) : stage;
}

export function localizeProgressMessage(t: Translate, scope: ProgressScope, message: string): string {
  const segment = message.match(/^Transcribing segment (\d+) of (\d+) \(([\d.]+)s\)\.\.\.$/);
  if (segment) {
    return t(`${scope}.transcribingSegment`, {
      current: Number(segment[1]),
      total: Number(segment[2]),
      seconds: segment[3],
    });
  }

  const speechProgress = message.match(/^Detecting speech segments\.\.\. (\d+)% \((\d+) found\)$/);
  if (speechProgress) {
    return t(`${scope}.detectingSpeechProgress`, {
      percent: Number(speechProgress[1]),
      count: Number(speechProgress[2]),
    });
  }

  if (message.startsWith('Decoding')) return t(`${scope}.decodingAudio`);
  if (message.startsWith('Converting')) return t(`${scope}.convertingAudio`);
  if (message.startsWith('Detecting speech segments')) return t(`${scope}.detectingSpeech`);

  const fixedMessages: Record<string, string> = {
    'Creating meeting folder...': `${scope}.creatingMeetingFolder`,
    'Copying audio file...': `${scope}.copyingAudio`,
    'Decoding audio file...': `${scope}.decodingAudio`,
    'Converting audio format...': `${scope}.convertingAudio`,
    'Detecting speech segments...': `${scope}.detectingSpeech`,
    'Loading transcription engine...': `${scope}.loadingEngine`,
    'Creating meeting...': `${scope}.creatingMeeting`,
    'Saving transcripts...': `${scope}.savingTranscripts`,
    'Writing transcript files...': `${scope}.writingTranscripts`,
    'Import complete': 'import.complete',
    'Retranscription complete': 'retranscribe.complete',
  };

  const key = fixedMessages[message];
  return key ? t(key) : message;
}
