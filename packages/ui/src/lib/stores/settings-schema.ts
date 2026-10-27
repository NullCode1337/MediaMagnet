import { DEFAULT_CONFIG, type Config } from "@mediamagnet/api";
import type { AccentKind } from "$lib/color";

export type FieldProblem =
  | "missing"
  | "wrong_type"
  | "out_of_range"
  | "bad_entry"
  | "unknown_key"
  | "future_version";

export type FieldIssue = {
  key: string;
  problem: FieldProblem;
  message: string;
};

type FieldRule =
  | { kind: "boolean" }
  | { kind: "string" }
  | { kind: "hue" }
  | { kind: "accent_kind" }
  | { kind: "version" }
  | { kind: "site_args" };

const FIELD_RULES: Record<keyof Config, FieldRule> = {
  settings_version: { kind: "version" },

  dark_mode: { kind: "boolean" },
  amoled_mode: { kind: "boolean" },
  accent_hue: { kind: "hue" },
  accent_kind: { kind: "accent_kind" },
  always_on_top: { kind: "boolean" },
  custom_titlebar: { kind: "boolean" },
  custom_titlebar_type: { kind: "string" },
  native_notifications: { kind: "boolean" },
  clear_cookies_on_exit: { kind: "boolean" },

  download_path: { kind: "string" },
  user_agent: { kind: "string" },
  preserve_mtime: { kind: "boolean" },

  yt_format: { kind: "string" },
  yt_recode_format: { kind: "string" },
  yt_output_template: { kind: "string" },
  yt_embed_thumbnail: { kind: "boolean" },
  yt_embed_subs: { kind: "boolean" },
  yt_restrict_filenames: { kind: "boolean" },
  yt_global_args: { kind: "string" },
  yt_site_args: { kind: "site_args" },

  gdl_global_args: { kind: "string" },
  gdl_site_args: { kind: "site_args" },

  spotdl_format: { kind: "string" },
  spotdl_bitrate: { kind: "string" },
  spotdl_global_args: { kind: "string" },
};

function describeType(value: unknown): string {
  if (value === null) return "null";
  if (Array.isArray(value)) return "array";
  return typeof value;
}

function isAccentKind(value: unknown): value is AccentKind {
  return value === "hue" || value === "black" || value === "white";
}

/**
 * Coerces whatever the backend sent into a complete `Config`. A server answer omits
 * desktop-only keys; those surface as `missing` issues, which the caller can tell
 * apart from a genuine corruption via `isDesktopKey`.
 */
export function normalizeConfig(raw: unknown): {
  config: Config;
  issues: FieldIssue[];
} {
  const config = { ...DEFAULT_CONFIG } as unknown as Record<string, unknown>;
  const issues: FieldIssue[] = [];

  if (raw === null || typeof raw !== "object" || Array.isArray(raw)) {
    issues.push({
      key: "<root>",
      problem: "wrong_type",
      message: "input is not a settings object — using defaults",
    });
    return { config: config as unknown as Config, issues };
  }

  const rawObj = raw as Record<string, unknown>;

  for (const key of Object.keys(DEFAULT_CONFIG) as (keyof Config)[]) {
    const value = rawObj[key];
    const rule = FIELD_RULES[key];

    if (value === undefined) {
      issues.push({
        key,
        problem: "missing",
        message: `${key}: missing — using default`,
      });
      continue;
    }

    switch (rule.kind) {
      case "boolean": {
        if (typeof value === "boolean") {
          config[key] = value;
        } else {
          issues.push({
            key,
            problem: "wrong_type",
            message: `${key}: expected boolean, got ${describeType(value)} — using ${String(DEFAULT_CONFIG[key])}`,
          });
        }
        break;
      }

      case "string": {
        if (typeof value === "string") {
          config[key] = value;
        } else {
          issues.push({
            key,
            problem: "wrong_type",
            message: `${key}: expected string, got ${describeType(value)} — using default`,
          });
        }
        break;
      }

      case "hue": {
        if (typeof value === "number" && Number.isFinite(value)) {
          const clamped = Math.min(360, Math.max(0, Math.round(value)));
          if (clamped !== value) {
            issues.push({
              key,
              problem: "out_of_range",
              message: `${key}: ${value} → ${clamped} (valid range 0–360)`,
            });
          }
          config.accent_hue = clamped;
        } else {
          issues.push({
            key,
            problem: "wrong_type",
            message: `${key}: expected number, got ${describeType(value)} — using ${DEFAULT_CONFIG.accent_hue}`,
          });
        }
        break;
      }

      case "accent_kind": {
        if (isAccentKind(value)) {
          config.accent_kind = value;
        } else {
          issues.push({
            key,
            problem: "wrong_type",
            message: `${key}: expected "hue", "black" or "white", got ${describeType(value)} — using "hue"`,
          });
        }
        break;
      }

      case "version": {
        if (typeof value === "number" && Number.isInteger(value) && value >= 1) {
          config.settings_version = value;
          if (value > DEFAULT_CONFIG.settings_version) {
            issues.push({
              key,
              problem: "future_version",
              message: `written by a newer app version (v${value}); unknown fields dropped, missing fields defaulted`,
            });
          }
        } else {
          issues.push({
            key,
            problem: "wrong_type",
            message: `${key}: expected positive integer, got ${describeType(value)}`,
          });
        }
        break;
      }

      case "site_args": {
        if (!Array.isArray(value)) {
          issues.push({
            key,
            problem: "wrong_type",
            message: `${key}: expected array, got ${describeType(value)} — using []`,
          });
          break;
        }
        const list: { id: string; domain: string; args: string }[] = [];
        value.forEach((entry, index) => {
          const obj = entry as Record<string, unknown> | null;
          if (
            obj !== null &&
            typeof obj === "object" &&
            typeof obj.domain === "string" &&
            typeof obj.args === "string"
          ) {
            // Rust assigns stable ids; only invent one when the source has none.
            const id =
              typeof obj.id === "string" && obj.id.length > 0
                ? obj.id
                : crypto.randomUUID();
            list.push({ id, domain: obj.domain, args: obj.args });
          } else {
            issues.push({
              key: `${key}[${index}]`,
              problem: "bad_entry",
              message: `${key}[${index}]: entry needs string domain and args — dropped`,
            });
          }
        });
        config[key] = list;
        break;
      }
    }
  }

  for (const key of Object.keys(rawObj)) {
    if (!(key in DEFAULT_CONFIG)) {
      issues.push({
        key,
        problem: "unknown_key",
        message: `${key}: unknown setting — dropped`,
      });
    }
  }

  return { config: config as unknown as Config, issues };
}
