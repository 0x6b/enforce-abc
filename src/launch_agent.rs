use std::{
    fs::{remove_file, write},
    io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use cmd_lib::{run_cmd, run_fun};
use dirs::home_dir;
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
        let uid = run_fun!(/usr/bin/id -u)?;
        let home = home_dir().context("could not determine home directory")?;
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

        log_cmd(run_cmd!(launchctl bootstrap gui/$uid $plist), "bootstrap");
        log_cmd(run_cmd!(launchctl load -w $plist), "load");
        log_cmd(run_cmd!(launchctl enable gui/$uid/$label), "enable");
        log_cmd(run_cmd!(launchctl start $label), "start");
        Ok(())
    }

    pub fn unregister(&self) -> Result<()> {
        let Self { label, plist, .. } = self;

        log_cmd(run_cmd!(launchctl stop $label), "stop");
        log_cmd(run_cmd!(launchctl unload -w $plist), "unload");

        match remove_file(plist) {
            Ok(()) => info!("Removed {plist:?}"),
            Err(why) => warn!("Failed to remove {plist:?}: {why}"),
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

fn log_cmd(result: io::Result<()>, cmd: &str) {
    match result {
        Ok(()) => info!("launchctl {cmd}: success"),
        Err(why) => warn!("launchctl {cmd}: {why}"),
    }
}
