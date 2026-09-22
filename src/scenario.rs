use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub(crate) enum ScenarioError {
    FileNotFound,
    ProcessFailed(std::io::Error),
    ParseFailed(String),
    InvalidOutput(serde_json::Error),
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScenarioError::FileNotFound => {
                write!(f, "Scenario file does not exist")
            }
            ScenarioError::ProcessFailed(error) => {
                write!(f, "Unable to execute parser: {}", error)
            }
            ScenarioError::ParseFailed(error) => {
                write!(f, "{}", error)
            }
            ScenarioError::InvalidOutput(error) => {
                write!(f, "Parser returned invalid output: {}", error)
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct ScenarioInfo {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

pub(crate) async fn parse_scenario(path: &Path) -> Result<ScenarioInfo, ScenarioError> {
    if !path.is_file() {
        return Err(ScenarioError::FileNotFound);
    }

    let python = std::env::var("AOE2SCENARIO_PYTHON").unwrap_or_else(|_| "python3".to_string());
    let output = tokio::process::Command::new(python)
        .arg("python/parse_scenario.py")
        .arg(path)
        .output()
        .await
        .map_err(ScenarioError::ProcessFailed)?;
    if !output.status.success() {
        return Err(ScenarioError::ParseFailed(format!(
            "Python parser failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let scenario: ScenarioInfo =
        serde_json::from_slice(&output.stdout).map_err(ScenarioError::InvalidOutput)?;
    Ok(scenario)
}
