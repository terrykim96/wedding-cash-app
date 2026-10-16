// Rust model.rs 와 같은 구조의 공유 타입 (JSON camelCase)

export type Side = "groom" | "bride";
export type LedgerKind = "groom" | "bride" | "merged";
export type HistoryKind = "update" | "delete" | "restore";
export type SortBy = "name" | "amount" | "receivedAt";
export type SortDir = "asc" | "desc";

export interface EntryInput {
  name: string;
  amountWon: number;
  side: Side;
  tickets: number;
  memo: string;
}

export interface Entry extends EntryInput {
  id: string;
  receivedAt: string;
  updatedAt: string;
  deleted: boolean;
  sourcePc: string;
}

export interface HistoryItem {
  id: string;
  entryId: string;
  kind: HistoryKind;
  at: string;
  before: EntryInput;
  after: EntryInput | null;
  pcName: string;
  entryName: string;
  canRestore: boolean;
}

export interface SideSummary {
  amountWon: number;
  count: number;
  tickets: number;
}

export interface Summary {
  total: SideSummary;
  groom: SideSummary;
  bride: SideSummary;
}

export interface EntryQuery {
  search?: string;
  side?: Side;
  sortBy: SortBy;
  sortDir: SortDir;
}

export interface QueryResult {
  entries: Entry[];
  filteredSummary: Summary;
  totalSummary: Summary;
}

export interface MergeReport {
  outputPath: string;
  fromA: number;
  fromB: number;
  merged: number;
  duplicatesResolved: number;
}

export interface AppStateView {
  needsSetup: boolean;
  pcSide: Side | null;
  pcName: string;
  defaultSide: Side;
  dataDir: string;
  activeFilePath: string | null;
  activeFileKind: LedgerKind | null;
  storageNotice: string | null;
  loadError: string | null;
  tutorialSeen: boolean;
}

/** 입력 오류 필드 이름 (Rust validation.rs 의 키와 동일) */
export type FieldName = "name" | "amount" | "tickets" | "memo";
export type FieldErrors = Partial<Record<FieldName, string>>;

export interface AppError {
  code: "validation" | "io" | "parse" | "notFound" | "state";
  message: string;
  fieldErrors?: FieldErrors;
}

export const SIDE_LABEL: Record<Side, string> = {
  groom: "신랑측",
  bride: "신부측",
};
