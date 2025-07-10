use std::fs;
use std::io;
use std::process::Command;
#[cfg(target_family = "unix")]
fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(target_family = "windows")]
fn is_elevated() -> bool {
    use std::ptr;
    use winapi::um::shellapi::IsUserAnAdmin;
    unsafe { IsUserAnAdmin() != 0 }
}

fn restart_with_admin() {
    #[cfg(target_family = "unix")]
    {
        let args: Vec<String> = std::env::args().collect();
        let status = Command::new("sudo")
            .args(&args)
            .status()
            .expect("failed to execute sudo");
        std::process::exit(status.code().unwrap_or(1));
    }
    #[cfg(target_family = "windows")]
    {
        use std::env;
        let exe = env::current_exe().unwrap();
        let args: Vec<String> = env::args().skip(1).collect();
        Command::new("powershell")
            .arg("-Command")
            .arg(format!(
                "Start-Process -Verb runAs -FilePath '{}' -ArgumentList '{}'",
                exe.display(),
                args.join(" ")
            ))
            .status()
            .expect("failed to execute runas");
        std::process::exit(0);
    }
}

fn delete_temp_folders() -> io::Result<()> {
    let temp_dir = std::env::temp_dir();
    for entry in fs::read_dir(&temp_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Attempt to remove the directory and all its contents
            if let Err(e) = fs::remove_dir_all(&path) {
                eprintln!("Failed to remove {:?}: {}", path, e);
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    if !is_elevated() {
        println!("Restarting with admin privileges...");
        restart_with_admin();
        return Ok(());
    }

    delete_temp_folders()?;
    Ok(())
}