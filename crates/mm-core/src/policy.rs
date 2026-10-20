use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgTool {
    YtDlp,
    GalleryDl,
    SpotDl,
}

impl ArgTool {
    pub fn name(self) -> &'static str {
        match self {
            ArgTool::YtDlp => "yt-dlp",
            ArgTool::GalleryDl => "gallery-dl",
            ArgTool::SpotDl => "spotdl",
        }
    }

    fn denied(self) -> &'static [Deny] {
        match self {
            ArgTool::YtDlp => YT_DLP_DENY,
            ArgTool::GalleryDl => GALLERY_DL_DENY,
            ArgTool::SpotDl => SPOTDL_DENY,
        }
    }
}

struct Deny {
    long: &'static [&'static str],
    short: &'static [char],
}

const EXEC_YT: Deny = Deny {
    long: &[
        "--exec",
        "--load-info-json",
        "--batch-file",
        "--config-locations",
        "--config",
        "--plugin-dirs",
        "--netrc-cmd",
        "--cookies-from-browser",
        "--output",
        "--paths",
        "--download-archive",
        "--print-to-file",
    ],
    short: &['a', 'o', 'P', 'c'],
};

const YT_DLP_DENY: &[Deny] = &[EXEC_YT];

const GALLERY_DL_DENY: &[Deny] = &[Deny {
    long: &[
        "--exec",
        "--exec-after",
        "--postprocessor",
        "--postprocessor-option",
        "--config",
        "--config-json",
        "--config-yaml",
        "--config-toml",
        "--destination",
        "--directory",
        "--option",
    ],
    short: &['c', 'P', 'O', 'd', 'D', 'o'],
}];

const SPOTDL_DENY: &[Deny] = &[Deny {
    long: &["--output", "--cookie-file", "--config"],
    short: &['o'],
}];

fn is_denied(tool: ArgTool, token: &str) -> bool {
    if token == "--" {
        return false;
    }
    tool.denied().iter().any(|rule| {
        rule.long
            .iter()
            .any(|long| token == *long || token.starts_with(&format!("{long}=")))
            || (token.starts_with('-')
                && !token.starts_with("--")
                && rule.short.iter().any(|short| {
                    let mut chars = token.chars();
                    chars.next() == Some('-') && chars.next() == Some(*short)
                }))
    })
}

/// What a caller is allowed to influence. Desktop is unrestricted; server mode keeps
/// user-supplied arguments away from file system and process escape hatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    pub allow_custom_args: bool,
    pub allow_client_paths: bool,
}

impl Policy {
    pub fn desktop() -> Self {
        Self {
            allow_custom_args: true,
            allow_client_paths: true,
        }
    }

    pub fn server() -> Self {
        Self {
            allow_custom_args: false,
            allow_client_paths: false,
        }
    }

    /// Server policy with `MM_ALLOW_CUSTOM_ARGS` enabled: user arguments are accepted
    /// but still filtered through the deny-list.
    pub fn server_with_custom_args() -> Self {
        Self {
            allow_custom_args: true,
            allow_client_paths: false,
        }
    }

    /// Splits and filters a custom argument string for one backend.
    pub fn filter_custom_args(&self, tool: ArgTool, args: &str) -> Result<Vec<String>> {
        if args.trim().is_empty() {
            return Ok(Vec::new());
        }
        if !self.allow_custom_args {
            return Err(Error::Msg(format!(
                "custom {} arguments are disabled on this server",
                tool.name()
            )));
        }
        let tokens = shell_words::split(args)
            .map_err(|_| Error::Msg("Unable to split the arguments!".to_string()))?;
        if self.allow_client_paths {
            return Ok(tokens);
        }
        for token in &tokens {
            if is_denied(tool, token) {
                return Err(Error::Msg(format!(
                    "argument '{}' is not allowed for {}",
                    token,
                    tool.name()
                )));
            }
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_allows_everything() {
        let policy = Policy::desktop();
        let args = policy
            .filter_custom_args(ArgTool::YtDlp, "--exec echo {}")
            .unwrap();
        assert_eq!(args, vec!["--exec", "echo", "{}"]);
    }

    #[test]
    fn server_with_custom_args_still_filters() {
        let policy = Policy::server_with_custom_args();
        assert_eq!(
            policy
                .filter_custom_args(ArgTool::YtDlp, "--no-mtime")
                .unwrap(),
            vec!["--no-mtime"]
        );
        assert!(policy
            .filter_custom_args(ArgTool::YtDlp, "--paths /tmp")
            .is_err());
    }

    #[test]
    fn server_refuses_custom_args_entirely() {
        let policy = Policy::server();
        assert!(policy
            .filter_custom_args(ArgTool::YtDlp, "--no-mtime")
            .is_err());
        assert!(policy
            .filter_custom_args(ArgTool::YtDlp, "   ")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn deny_list_catches_separate_equals_and_attached_forms() {
        let check = |tool, arg| is_denied(tool, arg);
        assert!(check(ArgTool::YtDlp, "--exec"));
        assert!(check(ArgTool::YtDlp, "--exec=rm -rf /"));
        assert!(check(ArgTool::YtDlp, "-o"));
        assert!(check(ArgTool::YtDlp, "-o/tmp/x"));
        assert!(check(ArgTool::YtDlp, "--paths"));
        assert!(check(ArgTool::YtDlp, "-P"));
        assert!(check(ArgTool::YtDlp, "--config-locations"));
        assert!(check(ArgTool::YtDlp, "--cookies-from-browser"));
        assert!(check(ArgTool::GalleryDl, "--postprocessor"));
        assert!(check(ArgTool::GalleryDl, "-D"));
        assert!(check(ArgTool::GalleryDl, "-c"));
        assert!(!check(ArgTool::YtDlp, "--no-mtime"));
        assert!(!check(ArgTool::YtDlp, "-f"));
        assert!(!check(ArgTool::GalleryDl, "--no-mtime"));
        assert!(!check(ArgTool::GalleryDl, "-R"));
    }
}
