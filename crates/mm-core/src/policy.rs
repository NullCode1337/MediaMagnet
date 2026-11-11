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
        "--config",
        "--plugin-dirs",
        "--netrc-cmd",
        "--cookies",
        "--cookies-from-browser",
        "--output",
        "--paths",
        "--download-archive",
        "--print-to-file",
        "--ffmpeg-location",
        "--downloader",
        "--postprocessor-args",
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
        "--destination",
        "--directory",
        "--option",
        "--cookies",
        "--filter",
        "--chapter-filter",
        "--image-filter",
        "--post-filter",
        "--write-log",
        "--write-unsupported",
    ],
    short: &['c', 'P', 'O', 'd', 'D', 'o', 'f'],
}];

const SPOTDL_DENY: &[Deny] = &[Deny {
    long: &["--output", "--cookie-file", "--config", "--ffmpeg"],
    short: &['o'],
}];

fn is_denied_long(token: &str, long: &str) -> bool {
    match token.strip_prefix(long) {
        Some(rest) => rest.is_empty() || rest.starts_with('=') || rest.starts_with('-'),
        None => false,
    }
}

#[doc(hidden)]
pub fn is_denied(tool: ArgTool, token: &str) -> bool {
    if token == "--" {
        return false;
    }
    tool.denied().iter().any(|rule| {
        rule.long.iter().any(|long| is_denied_long(token, long))
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
    pub cookie_store: bool,
}

impl Policy {
    pub fn desktop() -> Self {
        Self {
            allow_custom_args: true,
            allow_client_paths: true,
            cookie_store: true,
        }
    }

    pub fn server() -> Self {
        Self {
            allow_custom_args: false,
            allow_client_paths: false,
            cookie_store: false,
        }
    }

    /// Server policy with `MM_ALLOW_CUSTOM_ARGS` enabled: user arguments are accepted
    /// but still filtered through the deny-list.
    pub fn server_with_custom_args() -> Self {
        Self {
            allow_custom_args: true,
            allow_client_paths: false,
            cookie_store: false,
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
