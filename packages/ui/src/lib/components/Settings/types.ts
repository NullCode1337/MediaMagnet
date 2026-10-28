import type { Snippet } from "svelte";

export type Preset = { label: string; value: string };
export type Option = { label: string; value: string; icon?: Snippet<[]> };

export type GlobalArg = { id: string; domain: string; args: string };

export type SiteArgsItem = {
  type: "site-args";
  id?: string;
  value: GlobalArg[];
  onchange: (val: GlobalArg[]) => void;
  domainPlaceholder?: string;
  argsPlaceholder?: string;
};

export type SwitchItem = {
  type: "switch";
  id: string;
  label: string;
  description?: string;
  icon?: Snippet<[]>;
  value: boolean;
  onchange: (val: boolean) => void;
  disabled?: boolean;
};

export type InputItem = {
  type: "input";
  id: string;
  label: string;
  description?: string;
  value: string;
  onchange: (val: string) => void;
  placeholder?: string;
  inputType?: string;
  monospace?: boolean;
  disabled?: boolean;
};

export type TextareaItem = {
  type: "textarea";
  id: string;
  label: string;
  description?: string;
  descriptionNode?: Snippet<[]>;
  value: string;
  onchange: (val: string) => void;
  placeholder?: string;
  presets?: Preset[];
  monospace?: boolean;
  minRows?: number;
};

export type SegmentedItem = {
  type: "segmented";
  label?: string;
  description?: string;
  options: Option[];
  value: string;
  onchange: (val: string) => void;
};

export type ListEntry = {
  id: string;
  label: string;
  supporting?: string;
  icon?: Snippet<[]>;
  onclick?: () => void;
  ondelete?: () => void;
};

export type ListBlock = {
  type: "list";
  entries: ListEntry[];
  emptyMessage?: string;
  emptyIcon?: Snippet<[]>;
  maxHeight?: string;
};

export type CustomItem = {
  type: "custom";
  node: Snippet<[]>;
};

export type Setting =
  | SwitchItem
  | InputItem
  | TextareaItem
  | SegmentedItem
  | ListBlock
  | CustomItem
  | SiteArgsItem;

export type SectionAction = {
  label: string;
  onclick: () => void;
  variant?: "default" | "outline" | "destructive" | "ghost";
  icon?: Snippet<[]>;
};

export type PageAction = {
  label: string;
  onclick: () => void;
  variant?: "default" | "outline" | "destructive" | "ghost";
  icon?: Snippet<[]>;
};

export type Section = {
  label?: string;
  description?: string;
  headerAction?: SectionAction;
  items: Setting[];
};

export type SectionBase = {
  title: string;
  description?: string;
  headerActions?: PageAction[];
  sections: Section[];
};
