import { describe, it, expect } from 'vitest'
import { cn, truncate, formatCost } from '@/lib/utils'

describe('cn', () => {
  it('joins truthy classes and drops falsy ones', () => {
    expect(cn('a', false && 'b', 'c')).toBe('a c')
  })
})

describe('truncate', () => {
  it('leaves short strings unchanged', () => {
    expect(truncate('hello', 10)).toBe('hello')
  })
  it('truncates long strings with an ellipsis to maxLen', () => {
    const out = truncate('hello world', 8)
    expect(out).toBe('hello...')
    expect(out.length).toBe(8)
  })
})

describe('formatCost', () => {
  it('shows sub-cent costs as a floor', () => {
    expect(formatCost(0.0005)).toBe('<$0.001')
  })
  it('formats normal costs to 3 decimals', () => {
    expect(formatCost(0.123)).toBe('$0.123')
  })
})
