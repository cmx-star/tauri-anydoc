export interface ConvertResult {
  file_path: string;
  file_name: string;
  extension: string;
  file_size: number;
  success: boolean;
  markdown: string;
  error_message: string | null;
  engine: string;
  duration_ms: number;
}

export interface BatchConvertSummary {
  total: number;
  success_count: number;
  failure_count: number;
  results: ConvertResult[];
}

export type ConvertStatus = "idle" | "converting" | "success" | "error";

export interface TaskItem extends ConvertResult {
  id: string;
  status: ConvertStatus;
}
