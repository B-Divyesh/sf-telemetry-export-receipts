import { describe, expect, it } from 'vitest'
import { createArchive } from './archive'

describe('createArchive', () => {
  it('keeps signatures and records the item count', () => {
    const archive = createArchive([{ id: 'r-1', signature: 'signed' }], new Date('2026-08-28T00:00:00Z'))
    expect(archive).toEqual({
      schema: 'telemetry-export-archive.v1',
      generated_at: '2026-08-28T00:00:00.000Z',
      receipt_count: 1,
      receipts: [{ id: 'r-1', signature: 'signed' }],
    })
  })
})
