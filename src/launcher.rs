use std::collections::HashMap;
use std::path::Path;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use sysinfo::System;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS,
    NORMAL_PRIORITY_CLASS, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
    REALTIME_PRIORITY_CLASS, SetPriorityClass,
};

use crate::models::{LaunchProfile, LaunchTarget, PriorityClass};

pub fn launch(target: &LaunchTarget, profile: &LaunchProfile) -> Result<Child> {
    let mut command = Command::new(&target.executable);
    command.args(&target.args);
    command.current_dir(&target.working_dir);
    command.envs(merged_env(&target.env, &profile.env_overrides));
    let child = command
        .spawn()
        .with_context(|| format!("failed to launch {}", target.executable.display()))?;
    apply_process_tweaks(child.id(), profile)?;
    maybe_kill_background_processes(&profile.kill_after_launch)?;
    Ok(child)
}

fn merged_env(base: &HashMap<String, String>, override_env: &HashMap<String, String>) -> HashMap<String, String> {
    let mut merged = base.clone();
    for (key, value) in override_env {
        merged.insert(key.clone(), value.clone());
    }
    merged
}

fn apply_process_tweaks(pid: u32, profile: &LaunchProfile) -> Result<()> {
    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION, false, pid)
            .context("failed to open launched process for tuning")?;
        let priority = match profile.priority {
            PriorityClass::Idle => IDLE_PRIORITY_CLASS,
            PriorityClass::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
            PriorityClass::Normal => NORMAL_PRIORITY_CLASS,
            PriorityClass::AboveNormal => ABOVE_NORMAL_PRIORITY_CLASS,
            PriorityClass::High => HIGH_PRIORITY_CLASS,
            PriorityClass::Realtime => REALTIME_PRIORITY_CLASS,
        };
        SetPriorityClass(handle, priority).ok().context("failed to set process priority")?;
        if let Some(mask) = profile.affinity_mask {
            let status = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!("$p = Get-Process -Id {pid}; $p.ProcessorAffinity = {mask}"),
                ])
                .status()
                .context("failed to set process affinity")?;
            if !status.success() {
                CloseHandle(handle)?;
                anyhow::bail!("failed to set process affinity");
            }
        }
        CloseHandle(handle)?;
    }
    Ok(())
}

fn maybe_kill_background_processes(kill_list: &[String]) -> Result<()> {
    if kill_list.is_empty() {
        return Ok(());
    }
    thread::sleep(Duration::from_secs(2));
    let protected = ["explorer.exe", "svchost.exe", "wininit.exe", "services.exe", "lsass.exe"];
    let mut system = System::new_all();
    system.refresh_all();
    for target in kill_list {
        let target_lower = target.to_ascii_lowercase();
        if protected.contains(&target_lower.as_str()) {
            continue;
        }
        for process in system.processes().values() {
            let name = process.name().to_ascii_lowercase();
            if name == target_lower {
                let _ = process.kill();
            }
        }
    }
    Ok(())
}

pub fn build_export_script(target: &LaunchTarget, profile: &LaunchProfile, output_path: &Path) -> Result<String> {
    let env_lines = target
        .env
        .iter()
        .chain(profile.env_overrides.iter())
        .map(|(key, value)| format!("$env:{key} = '{}'", escape_ps(value)))
        .collect::<Vec<_>>()
        .join("; ");
    let args = target
        .args
        .iter()
        .map(|arg| format!("'{}'", escape_ps(arg)))
        .collect::<Vec<_>>()
        .join(", ");
    let kill_block = if profile.kill_after_launch.is_empty() {
        String::new()
    } else {
        format!(
            "Start-Sleep -Seconds 2; {}",
            profile
                .kill_after_launch
                .iter()
                .map(|name| format!("Get-Process -Name '{}' -ErrorAction SilentlyContinue | Stop-Process -Force", strip_exe(name)))
                .collect::<Vec<_>>()
                .join("; ")
        )
    };
    let affinity_block = profile
        .affinity_mask
        .map(|mask| format!("; $p.ProcessorAffinity = {mask}"))
        .unwrap_or_default();
    let priority = priority_name(&profile.priority);
    let script = format!(
        "@echo off\r\npowershell -NoProfile -ExecutionPolicy Bypass -Command \"{env_lines}; $p = Start-Process -FilePath '{exe}' -WorkingDirectory '{cwd}' -ArgumentList @({args}) -Priority {priority} -PassThru{affinity_block}; {kill_block}\"",
        exe = escape_ps(&target.executable.to_string_lossy()),
        cwd = escape_ps(&target.working_dir.to_string_lossy()),
    );
    std::fs::write(output_path, &script)?;
    Ok(script)
}

fn priority_name(priority: &PriorityClass) -> &'static str {
    match priority {
        PriorityClass::Idle => "Idle",
        PriorityClass::BelowNormal => "BelowNormal",
        PriorityClass::Normal => "Normal",
        PriorityClass::AboveNormal => "AboveNormal",
        PriorityClass::High => "High",
        PriorityClass::Realtime => "RealTime",
    }
}

fn strip_exe(value: &str) -> String {
    value.trim_end_matches(".exe").to_string()
}

fn escape_ps(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::build_export_script;
    use crate::models::{GameSource, LaunchProfile, LaunchTarget, LauncherMetadata, PriorityClass};

    #[test]
    fn export_contains_env_and_kill_rules() {
        let dir = tempdir().expect("tempdir");
        let output = dir.path().join("game.bat");
        let mut env = HashMap::new();
        env.insert("FOO".into(), "BAR".into());
        let target = LaunchTarget {
            executable: PathBuf::from(r"C:\Games\Halo\halo.exe"),
            args: vec!["-windowed".into()],
            working_dir: PathBuf::from(r"C:\Games\Halo"),
            env,
            source: GameSource::Local,
            metadata: LauncherMetadata::default(),
        };
        let profile = LaunchProfile {
            priority: PriorityClass::High,
            affinity_mask: Some(15),
            env_overrides: HashMap::new(),
            kill_after_launch: vec!["steam.exe".into()],
        };
        let script = build_export_script(&target, &profile, &output).expect("script");
        assert!(script.contains("$env:FOO"));
        assert!(script.contains("ProcessorAffinity = 15"));
        assert!(script.contains("Stop-Process"));
    }
}
