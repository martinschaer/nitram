export enum NitramErrorCode {
  DuplicateRequestQueued = "request already queued",
}

export class NitramError {
  error: NitramErrorCode;
  detail: Record<string, unknown>;
  constructor(error: NitramErrorCode, detail: Record<string, unknown>) {
    this.error = error;
    this.detail = detail;
  }
}
