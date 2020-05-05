use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::env::current_dir;
use std::time::{Duration, Instant};

use tokio::time;

use crate::IgniteResult;
use crate::IgniteError;

/// Simple abstraction to allow rust users find and run Ignite node.
/// Used for test purposes.
#[derive(Debug)]
pub struct IgniteNode {
    process: Option<Child>,
}

impl IgniteNode {
    /// Try to start a new instance of Ignite node.
    pub fn start<P: AsRef<OsStr>>(cfg_path: P) -> IgniteResult<Self> {
        let process = Some(start_process(cfg_path)?);
        Ok(Self{process})
    }

    /// Try to stop this node.
    pub fn stop(&mut self) -> IgniteResult<()> {
        match &mut self.process {
            None => {},
            Some(node) => {
                kill_process_tree(node.id())?;
                self.process = None;
            },
        };
        Ok(())
    }
}

impl Drop for IgniteNode {
    fn drop(&mut self) {
        let res = self.stop();
        match res {
            Ok(()) => println!("Killed successfully"),
            Err(err) => println!("Error during drop: {}", err),
        };
    }
}

/// Killing process tree with all it's children on Windows.
#[cfg(target_family = "windows")]
pub fn kill_process_tree(proc_id: u32) -> IgniteResult<()> {
    let res = Command::new("taskkill")
        .arg("/F")
        .arg("/T")
        .arg("/PID")
        .arg(format!("{}", proc_id))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;

    if res.success() {
        return Ok(());
    }

    Err(IgniteError::new(format!("Error while killing process: {}", res.to_string())))
}

/// Killing process tree with all it's children on Unix-like systems.
/// WARNING: Not tested yet
#[cfg(target_family = "unix")]
pub fn kill_process_tree(proc_id: u32) -> IgniteResult<()> {
    let res = Command::new("pkill")
        .arg("-9")
        .arg("-p")
        .arg(format!("{}", proc_id))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;

    if res.success() {
        return Ok(());
    }

    Err(IgniteError::new(format!("Error while killing process: {}", res.to_string())))
}

/// Start node for tests.
pub async fn start_test_node(cfg_name: &str) -> IgniteResult<IgniteNode> {
    let current_dir = current_dir().unwrap();
    let cfg = current_dir.join("tests").join("cfg").join(cfg_name);

    println!("Starting node");

    let node = IgniteNode::start(cfg)?;
    wait_till_available(10800u16, 20000u64).await?;

    Ok(node)
}

/// Wait until the node is available for connection by thin client.
pub async fn wait_till_available(port: u16, timeout: u64) -> IgniteResult<()> {
    use ignite_rust::IgniteClient;
    use ignite_rust::ClientConfiguration;

    let mut cfg = ClientConfiguration::new();
    cfg.set_endpoints(&format!("127.0.0.1:{}", port))?;

    let start_at = Instant::now();

    loop {
        let now = Instant::now();
        let elapsed = now - start_at;

        if elapsed > Duration::from_millis(timeout) {
            return Err(IgniteError::new("Timeout"));
        }

        time::delay_for(Duration::from_millis(1000)).await;

        if IgniteClient::start(cfg.clone()).await.is_ok() {
            return Ok(());
        }
    }
}

/// Locate and run OS-dependent script that starts Ignite node.
fn start_process<P: AsRef<OsStr>>(path: P) -> IgniteResult<Child> {
    let ignite_home = match env::var_os("IGNITE_HOME") {
        Some(home) => home,
        None => return Err(IgniteError::new("IGNITE_HOME is not set")),
    };

    let bin_path = Path::new(&ignite_home).join("bin").join("ignite");

    let script_path = if cfg!(target_os = "windows") {
        bin_path.with_extension("bat")
    } else {
        bin_path.with_extension("sh")
    };

    Ok(Command::new(script_path)
                .arg(&path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?)
}
