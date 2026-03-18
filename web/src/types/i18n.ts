export type LocalizedPrimitive = string | number | boolean | null | undefined;

export interface LocalizedText {
  key: string;
  values?: Record<string, LocalizedPrimitive>;
}
