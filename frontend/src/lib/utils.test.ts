import { describe, it, expect } from 'vitest';
import { cn } from '@/lib/utils';

describe('cn (className merger)', () => {
  it('merges single class', () => {
    expect(cn('foo')).toBe('foo');
  });

  it('merges multiple classes', () => {
    expect(cn('foo', 'bar')).toBe('foo bar');
  });

  it('handles conditional classes', () => {
    expect(cn('foo', false && 'bar', 'baz')).toBe('foo baz');
  });

  it('merges tailwind classes correctly', () => {
    // twMerge should handle conflicting tailwind classes
    expect(cn('px-4', 'px-8')).toBe('px-8');
  });

  it('handles undefined and null', () => {
    expect(cn('foo', undefined, null, 'bar')).toBe('foo bar');
  });

  it('handles empty input', () => {
    expect(cn()).toBe('');
  });
});
