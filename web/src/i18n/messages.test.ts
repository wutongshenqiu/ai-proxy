import { describe, expect, it } from 'vitest';
import { enMessages, pseudoMessages, zhMessages } from './messages';

describe('message catalogs', () => {
  it('keeps Chinese and pseudo catalogs aligned with English keys', () => {
    const englishKeys = Object.keys(enMessages).sort();
    expect(Object.keys(zhMessages).sort()).toEqual(englishKeys);
    expect(Object.keys(pseudoMessages).sort()).toEqual(englishKeys);
  });

  it('preserves interpolation placeholders in pseudo-localized strings', () => {
    expect(pseudoMessages['format.duration.ms']).toContain('{value}');
    expect(pseudoMessages['common.raw']).toContain('{value}');
  });

  it('expands visible strings in pseudo locale', () => {
    expect(pseudoMessages['common.controlPlane']).not.toBe(enMessages['common.controlPlane']);
    expect(pseudoMessages['common.controlPlane']).toMatch(/^\[!!/);
  });
});
