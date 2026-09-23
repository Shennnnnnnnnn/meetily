'use client';

import { useMemo, useState } from 'react';
import { Plus, Save, Trash2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { useI18n } from '@/i18n';

type SectionFormat = 'paragraph' | 'list' | 'string';

interface DraftSection {
  title: string;
  instruction: string;
  format: SectionFormat;
}

const DEFAULT_SECTION: DraftSection = {
  title: 'Summary',
  instruction: 'Summarize the key points, decisions, and context.',
  format: 'paragraph',
};

function slugify(value: string): string {
  const slug = value
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 48);
  if (slug) return `custom-${slug}-format`;

  // Keep user-defined Chinese (and other non-ASCII) names distinct while
  // retaining the filename-safe template ID required by the Rust loader.
  const codePointSlug = Array.from(value.trim())
    .map((character) => character.codePointAt(0)?.toString(16))
    .filter(Boolean)
    .join('')
    .slice(0, 48);
  return `custom-${codePointSlug || 'meeting'}-format`;
}

export function TemplateEditor() {
  const { t } = useI18n();
  const [name, setName] = useState('Chinese Meeting Summary');
  const [description, setDescription] = useState('A custom summary format for Chinese meetings.');
  const [sections, setSections] = useState<DraftSection[]>([DEFAULT_SECTION]);
  const [saving, setSaving] = useState(false);

  const templateId = useMemo(() => slugify(name), [name]);

  const updateSection = (index: number, patch: Partial<DraftSection>) => {
    setSections((current) => current.map((section, i) => i === index ? { ...section, ...patch } : section));
  };

  const handleSave = async () => {
    if (!name.trim() || sections.length === 0 || sections.some((section) => !section.title.trim() || !section.instruction.trim())) {
      toast.error(t('summary.templateRequired'));
      return;
    }

    setSaving(true);
    try {
      await invoke('api_save_template', {
        templateId,
        templateJson: JSON.stringify({
          name: name.trim(),
          description: description.trim() || name.trim(),
          sections,
        }),
      });
      window.dispatchEvent(new CustomEvent('summary-template-updated'));
      toast.success(t('summary.templateSaved'));
    } catch (error) {
      console.error('Failed to save custom template:', error);
      toast.error(t('summary.templateSaveFailed'), {
        description: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-4 rounded-lg border border-gray-200 bg-gray-50 p-4">
      <div>
        <h3 className="text-base font-semibold text-gray-900">{t('summary.customTemplate')}</h3>
        <p className="mt-1 text-sm text-gray-600">{t('summary.customTemplateDescription')}</p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2">
        <label className="space-y-1 text-sm font-medium text-gray-700">
          <span>{t('summary.templateName')}</span>
          <Input value={name} onChange={(event) => setName(event.target.value)} />
        </label>
        <label className="space-y-1 text-sm font-medium text-gray-700">
          <span>{t('summary.templateDescription')}</span>
          <Input value={description} onChange={(event) => setDescription(event.target.value)} />
        </label>
      </div>

      <div className="space-y-3">
        {sections.map((section, index) => (
          <div key={`${index}-${section.title}`} className="rounded-md border border-gray-200 bg-white p-3">
            <div className="mb-3 flex items-center justify-between gap-2">
              <h4 className="text-sm font-semibold text-gray-800">{t('summary.templateSection', { number: index + 1 })}</h4>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                onClick={() => setSections((current) => current.filter((_, i) => i !== index))}
                disabled={sections.length === 1}
                title={t('summary.removeSection')}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
            <div className="grid gap-3 sm:grid-cols-[1fr_150px]">
              <Input
                value={section.title}
                placeholder={t('summary.sectionTitle')}
                onChange={(event) => updateSection(index, { title: event.target.value })}
              />
              <select
                value={section.format}
                aria-label={t('summary.sectionFormat')}
                onChange={(event) => updateSection(index, { format: event.target.value as SectionFormat })}
                className="h-10 rounded-md border border-gray-300 bg-white px-3 text-sm"
              >
                <option value="paragraph">{t('summary.paragraph')}</option>
                <option value="list">{t('summary.list')}</option>
                <option value="string">{t('summary.string')}</option>
              </select>
            </div>
            <Textarea
              value={section.instruction}
              placeholder={t('summary.sectionInstruction')}
              onChange={(event) => updateSection(index, { instruction: event.target.value })}
              className="mt-3 min-h-20 resize-y"
            />
          </div>
        ))}
      </div>

      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="outline"
          onClick={() => setSections((current) => [...current, { ...DEFAULT_SECTION, title: '' }])}
        >
          <Plus className="mr-2 h-4 w-4" />
          {t('summary.addSection')}
        </Button>
        <Button type="button" onClick={handleSave} disabled={saving}>
          <Save className="mr-2 h-4 w-4" />
          {t('summary.saveTemplate')}
        </Button>
      </div>
    </div>
  );
}
