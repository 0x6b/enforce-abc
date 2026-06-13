use std::{
    env::var_os,
    fs::{remove_file, write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use log::{info, warn};

#[derive(Debug)]
pub struct LaunchAgent {
    label: String,
    uid: String,
    bin: PathBuf,
    plist: PathBuf,
}

impl LaunchAgent {
    pub fn new(label: &str) -> Result<Self> {
        let output = Command::new("/usr/bin/id")
            .arg("-u")
            .output()
            .context("failed to run id -u")?;
        let uid = String::from_utf8(output.stdout)
            .context("id -u output is not UTF-8")?
            .trim()
            .to_string();
        let home = var_os("HOME")
            .map(PathBuf::from)
            .context("HOME environment variable is not set")?;
        Ok(Self {
            label: label.to_string(),
            uid,
            bin: home.join(".cargo/bin").join(label),
            plist: home.join("Library/LaunchAgents").join(format!("{label}.plist")),
        })
    }

    pub fn register(&self) -> Result<()> {
        let Self { label, uid, bin, plist } = self;
        write(plist, plist_contents(label, bin))?;
        let plist = plist.to_string_lossy();

        launchctl(&["bootstrap", &format!("gui/{uid}"), &plist], "bootstrap");
        launchctl(&["load", "-w", &plist], "load");
        launchctl(&["enable", &format!("gui/{uid}/{label}")], "enable");
        launchctl(&["start", label], "start");
        Ok(())
    }

    pub fn unregister(&self) -> Result<()> {
        let Self { label, plist, .. } = self;
        let plist_str = plist.to_string_lossy();

        launchctl(&["stop", label], "stop");
        launchctl(&["unload", "-w", &plist_str], "unload");

        match remove_file(plist) {
            Ok(()) => info!("Removed {}", plist.display()),
            Err(why) => warn!("Failed to remove {}: {why}", plist.display()),
        }
        Ok(())
    }
}

fn plist_contents(label: &str, bin: &Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
    <dict>
        <key>Label</key>
        <string>{label}</string>
        <key>ProcessType</key>
        <string>Interactive</string>
        <key>ProgramArguments</key>
        <array>
            <string>{bin}</string>
        </array>
        <key>KeepAlive</key>
        <true/>
        <key>RunAtLoad</key>
        <true/>
        <key>StandardOutPath</key>
        <string>/tmp/{label}.out.log</string>
        <key>StandardErrorPath</key>
        <string>/tmp/{label}.err.log</string>
    </dict>
</plist>
"#,
        bin = bin.display(),
    )
}

fn launchctl(args: &[&str], cmd: &str) {
    match Command::new("launchctl").args(args).status() {
        Ok(status) if status.success() => info!("launchctl {cmd}: success"),
        Ok(status) => warn!("launchctl {cmd}: exited with {status}"),
        Err(why) => warn!("launchctl {cmd}: {why}"),
    }
}
