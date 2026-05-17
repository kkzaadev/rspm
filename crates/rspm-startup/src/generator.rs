//! Startup unit/script generator for systemd / OpenRC / SysV.
//!
//! Each branch produces a self-contained unit (or shell script) that runs
//! `rspm-daemon` (or whatever binary the caller passes) under the requested
//! user and home directory. The rendered text is returned to the CLI which
//! then writes it to the right location and enables it.

use std::path::PathBuf;

use crate::detect::InitSystem;

/// Startup generation context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupCtx {
    /// Service name (`rspm` by default).
    pub service: String,
    /// User that owns the service.
    pub user: String,
    /// Home directory for RSPM.
    pub rspm_home: PathBuf,
    /// Binary path to invoke as the daemon.
    pub binary: PathBuf,
}

impl StartupCtx {
    /// Convenience: build a default context for a given binary path.
    pub fn for_binary(binary: impl Into<PathBuf>) -> Self {
        Self {
            service: "rspm".to_owned(),
            user: current_user_name(),
            rspm_home: default_home(),
            binary: binary.into(),
        }
    }
}

/// Generates an init script or unit appropriate for the supplied init system.
pub fn generate(init: InitSystem, ctx: &StartupCtx) -> String {
    match init {
        InitSystem::Systemd => render_systemd(ctx),
        InitSystem::OpenRc => render_openrc(ctx),
        InitSystem::Sysv => render_sysv(ctx),
    }
}

fn render_systemd(ctx: &StartupCtx) -> String {
    format!(
        "[Unit]\n\
         Description=RSPM ({service})\n\
         After=network.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         User={user}\n\
         Environment=RSPM_HOME={home}\n\
         ExecStart={bin} --daemon\n\
         Restart=on-failure\n\
         KillSignal=SIGINT\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n",
        service = ctx.service,
        user = ctx.user,
        home = ctx.rspm_home.display(),
        bin = ctx.binary.display(),
    )
}

fn render_openrc(ctx: &StartupCtx) -> String {
    format!(
        "#!/sbin/openrc-run\n\
         name=\"{service}\"\n\
         description=\"RSPM process manager\"\n\
         command=\"{bin}\"\n\
         command_args=\"--daemon\"\n\
         command_user=\"{user}\"\n\
         pidfile=\"/run/{service}.pid\"\n\
         export RSPM_HOME=\"{home}\"\n\
         \n\
         depend() {{\n\
         \tneed net\n\
         }}\n",
        service = ctx.service,
        user = ctx.user,
        home = ctx.rspm_home.display(),
        bin = ctx.binary.display(),
    )
}

fn render_sysv(ctx: &StartupCtx) -> String {
    format!(
        "#!/bin/sh\n\
         ### BEGIN INIT INFO\n\
         # Provides:          {service}\n\
         # Required-Start:    $network\n\
         # Required-Stop:     $network\n\
         # Default-Start:     2 3 4 5\n\
         # Default-Stop:      0 1 6\n\
         # Short-Description: RSPM process manager\n\
         ### END INIT INFO\n\
         \n\
         RSPM_HOME=\"{home}\" exec \"{bin}\" --daemon\n",
        service = ctx.service,
        home = ctx.rspm_home.display(),
        bin = ctx.binary.display(),
    )
}

fn current_user_name() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "root".to_owned())
}

fn default_home() -> PathBuf {
    std::env::var_os("RSPM_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs_home_dir()
                .unwrap_or_else(|| PathBuf::from("/root"))
                .join(".rspm")
        })
}

fn dirs_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ctx() -> StartupCtx {
        StartupCtx {
            service: "rspm".into(),
            user: "deploy".into(),
            rspm_home: PathBuf::from("/var/lib/rspm"),
            binary: PathBuf::from("/usr/local/bin/rspm-daemon"),
        }
    }

    #[test]
    fn systemd_contains_required_directives() {
        let unit = generate(InitSystem::Systemd, &sample_ctx());
        assert!(unit.contains("[Unit]"));
        assert!(unit.contains("ExecStart=/usr/local/bin/rspm-daemon --daemon"));
        assert!(unit.contains("User=deploy"));
        assert!(unit.contains("Environment=RSPM_HOME=/var/lib/rspm"));
        assert!(unit.contains("WantedBy=multi-user.target"));
    }

    #[test]
    fn openrc_contains_shebang_and_command() {
        let script = generate(InitSystem::OpenRc, &sample_ctx());
        assert!(script.starts_with("#!/sbin/openrc-run"));
        assert!(script.contains("command=\"/usr/local/bin/rspm-daemon\""));
        assert!(script.contains("command_user=\"deploy\""));
    }

    #[test]
    fn sysv_contains_init_header() {
        let script = generate(InitSystem::Sysv, &sample_ctx());
        assert!(script.starts_with("#!/bin/sh"));
        assert!(script.contains("Provides:          rspm"));
        assert!(script.contains("exec \"/usr/local/bin/rspm-daemon\" --daemon"));
    }
}
