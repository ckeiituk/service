use anyhow::Error;

pub fn run_command(cmd: &str, args: &[&str], debug: bool) -> Result<(), Error> {
    if debug {
        println!("┌─────────────────────────────────────────");
        println!("│ Executing Command");
        println!("│ Command: {} {}", cmd, args.join(" "));
        println!("└─────────────────────────────────────────");
    }

    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute '{}': {}", cmd, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        if debug {
            eprintln!("\n┌─────────────────────────────────────────");
            eprintln!("│ Command Execution Failed");
            eprintln!("├─────────────────────────────────────────");
            eprintln!("│ Command: {} {}", cmd, args.join(" "));
            eprintln!("│ Status: {}", output.status);
            eprintln!("├─────────────────────────────────────────");
            if !stdout.is_empty() {
                eprintln!("│ STDOUT:");
                for line in stdout.lines() {
                    eprintln!("│   {}", line);
                }
            }
            if !stderr.is_empty() {
                eprintln!("├─────────────────────────────────────────");
                eprintln!("│ STDERR:");
                for line in stderr.lines() {
                    eprintln!("│   {}", line);
                }
            }
            eprintln!("└─────────────────────────────────────────\n");
        }

        return Err(anyhow::anyhow!(
            "Command execution failed:\n\
            Command: {} {}\n\
            Status: {}\n\
            stdout: {}\n\
            stderr: {}",
            cmd,
            args.join(" "),
            output.status,
            stdout,
            stderr
        ));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn uninstall_old_service() -> Result<(), Error> {
    use std::path::Path;

    fn try_launchctl(args: &[&str]) {
        let _ = run_command("launchctl", args, false);
    }

    let legacy_items = [
        (
            "io.github.clashverge.helper",
            "/Library/LaunchDaemons/io.github.clashverge.helper.plist",
        ),
        (
            "io.github.koala-clash.helper",
            "/Library/LaunchDaemons/io.github.koala-clash.helper.plist",
        ),
        (
            "io.github.koala-clash.service",
            "/Library/LaunchDaemons/io.github.koala-clash.service.plist",
        ),
    ];

    for (identifier, plist) in legacy_items {
        try_launchctl(&["stop", identifier]);
        try_launchctl(&["bootout", "system", plist]);
        let disable_target = format!("system/{}", identifier);
        try_launchctl(&["disable", &disable_target]);

        let plist_path = Path::new(plist);
        if plist_path.exists() {
            std::fs::remove_file(plist_path).map_err(|e| {
                anyhow::anyhow!("Failed to remove legacy launchd plist {}: {}", plist, e)
            })?;
        }
    }

    let legacy_binaries = [
        "/Library/PrivilegedHelperTools/io.github.clashverge.helper",
        "/Library/PrivilegedHelperTools/io.github.koala-clash.helper",
    ];

    for binary in legacy_binaries {
        let path = Path::new(binary);
        if path.exists() {
            std::fs::remove_file(path).map_err(|e| {
                anyhow::anyhow!("Failed to remove legacy helper binary {}: {}", binary, e)
            })?;
        }
    }

    let legacy_bundles =
        ["/Library/PrivilegedHelperTools/io.github.koala-clash.service.bundle"];

    for bundle in legacy_bundles {
        let path = Path::new(bundle);
        if path.exists() {
            std::fs::remove_dir_all(path).map_err(|e| {
                anyhow::anyhow!("Failed to remove legacy service bundle {}: {}", bundle, e)
            })?;
        }
    }

    Ok(())
}
