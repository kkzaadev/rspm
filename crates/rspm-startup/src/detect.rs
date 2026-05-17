//! Init system detection.
//!
//! Mirrors what `pm2 startup` does under the hood — pick the local init
//! system by probing filesystem markers, with a sensible fallback to SysV.

use std::path::{Path, PathBuf};

/// Supported init systems.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitSystem {
    /// systemd.
    Systemd,
    /// OpenRC.
    OpenRc,
    /// SysV init.
    Sysv,
}

impl InitSystem {
    /// Returns the short identifier (`systemd` / `openrc` / `sysv`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Systemd => "systemd",
            Self::OpenRc => "openrc",
            Self::Sysv => "sysv",
        }
    }

    /// Parses the short identifier into an [`InitSystem`].
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "systemd" => Some(Self::Systemd),
            "openrc" | "open-rc" | "open_rc" => Some(Self::OpenRc),
            "sysv" | "sysvinit" | "init" => Some(Self::Sysv),
            _ => None,
        }
    }

    /// Returns the default install path for a unit/script with `service` name.
    pub fn unit_path(self, service: &str) -> PathBuf {
        match self {
            Self::Systemd => PathBuf::from(format!("/etc/systemd/system/{service}.service")),
            Self::OpenRc | Self::Sysv => PathBuf::from(format!("/etc/init.d/{service}")),
        }
    }
}

/// Detects the current init system.
///
/// Checks (in order):
/// 1. `/run/systemd/system` → systemd
/// 2. `/sbin/openrc` or `/usr/sbin/openrc` → OpenRC
/// 3. fallback → SysV init
pub fn detect_init_system() -> InitSystem {
    if Path::new("/run/systemd/system").exists() {
        return InitSystem::Systemd;
    }
    if Path::new("/sbin/openrc").exists() || Path::new("/usr/sbin/openrc").exists() {
        return InitSystem::OpenRc;
    }
    InitSystem::Sysv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_one_of_known() {
        let init = detect_init_system();
        assert!(matches!(
            init,
            InitSystem::Systemd | InitSystem::OpenRc | InitSystem::Sysv
        ));
    }

    #[test]
    fn parse_round_trip() {
        for value in ["systemd", "openrc", "sysv"] {
            let parsed = InitSystem::parse(value).expect("known");
            assert_eq!(parsed.name(), value);
        }
        assert!(InitSystem::parse("upstart").is_none());
    }

    #[test]
    fn unit_path_matches_convention() {
        assert_eq!(
            InitSystem::Systemd.unit_path("rspm"),
            PathBuf::from("/etc/systemd/system/rspm.service")
        );
        assert_eq!(
            InitSystem::OpenRc.unit_path("rspm"),
            PathBuf::from("/etc/init.d/rspm")
        );
        assert_eq!(
            InitSystem::Sysv.unit_path("rspm"),
            PathBuf::from("/etc/init.d/rspm")
        );
    }
}
