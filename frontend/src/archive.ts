export type ArchiveItem = { id: string; signature: string }

export function createArchive<T extends ArchiveItem>(receipts: T[], at = new Date()) {
  return {
    schema: 'telemetry-export-archive.v1',
    generated_at: at.toISOString(),
    receipt_count: receipts.length,
    receipts,
  }
}
