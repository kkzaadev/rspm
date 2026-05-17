//! `rspm startup` and `rspm unstartup` commands.

use std::path::Path;
use std::process::Command;

use anyhow::{Result, anyhow};
use rspm_startup::{InitSystem, StartupCtx, detect_init_system, generate};

use crate::cli::StartupArgs;

/// Generates and installs (when running as root) the init script for the
/// detected system. Mirrors `pm2 startup`.
pub async fn install(args: StartupArgs) -> Result<()> {
    let init = resolve_init(&args)?;
    let ctx = build_ctx(&args)?;
    let unit_text = generate(init, &ctx);
    let target = init.unit_path(&ctx.service);

    if !is_root() {
        print_manual_steps(init, &target, &unit_text);
        return Ok(());
    }

    std::fs::create_dir_all(target.parent().unwrap_or_else(|| std::path::Path::new("/")))?;
    std::fs::write(&target, &unit_text)?;
    println!("[rspm] wrote {}", target.display());

    enable_service(init, &ctx.service).map_err(|err| {
        anyhow!(
            "wrote {target} but failed to enable service: {err}",
            target = target.display()
        )
    })?;
    println!("[rspm] enabled {} via {}", ctx.service, init.name());
    Ok(())
}

/// Disables and removes the installed init script. Mirrors `pm2 unstartup`.
pub async fn uninstall(args: StartupArgs) -> Result<()> {
    let init = resolve_init(&args)?;
    let ctx = build_ctx(&args)?;
    let target = init.unit_path(&ctx.service);

    if !is_root() {
        println!(
            "[rspm] re-run as root to remove {target}",
            target = target.display()
        );
        return Ok(());
    }

    disable_service(init, &ctx.service).ok();
    if target.exists() {
        std::fs::remove_file(&target)?;
        println!("[rspm] removed {}", target.display());
    } else {
        println!("[rspm] no unit at {}", target.display());
    }
    reload_units(init).ok();
    Ok(())
}

fn resolve_init(args: &StartupArgs) -> Result<InitSystem> {
    match args.platform.as_deref() {
        Some(value) => InitSystem::parse(value)
            .ok_or_else(|| anyhow!("unknown init system: {value}; expected systemd|openrc|sysv")),
        None => Ok(detect_init_system()),
    }
}

fn build_ctx(args: &StartupArgs) -> Result<StartupCtx> {
    let binary = std::env::current_exe().map_err(|err| anyhow!("locate current binary: {err}"))?;
    let mut ctx = StartupCtx::for_binary(binary);
    ctx.service = args.service.clone();
    if let Some(user) = args.user.as_deref() {
        ctx.user = user.to_owned();
    }
    Ok(ctx)
}

fn is_root() -> bool {
    nix::unistd::geteuid().is_root()
}

fn print_manual_steps(init: InitSystem, target: &Path, body: &str) {
    println!("[rspm] run the following as root to install the service:");
    println!(
        "[rspm] mkdir -p {}",
        target.parent().unwrap_or(target).display()
    );
    println!(
        "[rspm] tee {target} <<'RSPM-EOF'\n{body}RSPM-EOF",
        target = target.display(),
        body = body
    );
    match init {
        InitSystem::Systemd => {
            println!("[rspm] systemctl daemon-reload && systemctl enable --now rspm");
        }
        InitSystem::OpenRc => {
            println!(
                "[rspm] chmod +x {target} && rc-update add rspm default && rc-service rspm start",
                target = target.display()
            );
        }
        InitSystem::Sysv => {
            println!(
                "[rspm] chmod +x {target} && update-rc.d rspm defaults && service rspm start",
                target = target.display()
            );
        }
    }
}

fn enable_service(init: InitSystem, service: &str) -> Result<()> {
    match init {
        InitSystem::Systemd => run("systemctl", &["daemon-reload"])
            .and_then(|_| run("systemctl", &["enable", "--now", service])),
        InitSystem::OpenRc => run("rc-update", &["add", service, "default"])
            .and_then(|_| run("rc-service", &[service, "start"])),
        InitSystem::Sysv => run("update-rc.d", &[service, "defaults"])
            .and_then(|_| run("service", &[service, "start"])),
    }
}

fn disable_service(init: InitSystem, service: &str) -> Result<()> {
    match init {
        InitSystem::Systemd => run("systemctl", &["disable", "--now", service]),
        InitSystem::OpenRc => run("rc-update", &["del", service, "default"])
            .and_then(|_| run("rc-service", &[service, "stop"])),
        InitSystem::Sysv => run("update-rc.d", &["-f", service, "remove"])
            .and_then(|_| run("service", &[service, "stop"])),
    }
}

fn reload_units(init: InitSystem) -> Result<()> {
    if matches!(init, InitSystem::Systemd) {
        run("systemctl", &["daemon-reload"])?;
    }
    Ok(())
}

fn run(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("{cmd} {args:?} exited with {status}"))
    }
}
