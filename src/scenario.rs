use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::time::Duration;

const PARSER_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug)]
pub(crate) enum ScenarioError {
    FileNotFound,
    ProcessFailed(std::io::Error),
    ParseFailed(String),
    ProcessTerminated,
    ParserTimedOut,
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
            ScenarioError::ProcessTerminated => {
                write!(f, "Python parser was terminated unexpectedly")
            }
            ScenarioError::ParserTimedOut => {
                write!(
                    f,
                    "Python parser timed out after {} seconds",
                    PARSER_TIMEOUT.as_secs()
                )
            }
            ScenarioError::InvalidOutput(error) => {
                write!(f, "Parser returned invalid output: {}", error)
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
struct PlayerResources {
    food: i32,
    wood: i32,
    gold: i32,
    stone: i32,
}

#[derive(Deserialize, Serialize)]
struct PlayerInfo {
    id: u8,
    name: Option<String>,
    active: bool,
    human: bool,
    color: i32,
    civilization: String,
    starting_age: String,
    population_cap: Option<i32>,
    resources: PlayerResources,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct ScenarioInfo {
    pub(crate) width: u32,
    pub(crate) height: u32,

    #[serde(skip_serializing)]
    pub(crate) terrain: Vec<u32>,

    players: Vec<PlayerInfo>,
}

pub(crate) async fn parse_scenario(path: &Path) -> Result<ScenarioInfo, ScenarioError> {
    if !path.is_file() {
        return Err(ScenarioError::FileNotFound);
    }

    let python = std::env::var("AOE2SCENARIO_PYTHON").unwrap_or_else(|_| "python3".to_string());

    let output = tokio::time::timeout(
        PARSER_TIMEOUT,
        tokio::process::Command::new(python)
            .kill_on_drop(true)
            .arg("python/parse_scenario.py")
            .arg(path)
            .output(),
    )
    .await
    .map_err(|_| ScenarioError::ParserTimedOut)?
    .map_err(ScenarioError::ProcessFailed)?;

    match output.status.code() {
        Some(0) => {}
        Some(code) => {
            return Err(ScenarioError::ParseFailed(format!(
                "Python parser failed with status {}: {}",
                code,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        None => {
            return Err(ScenarioError::ProcessTerminated);
        }
    }

    let scenario: ScenarioInfo =
        serde_json::from_slice(&output.stdout).map_err(ScenarioError::InvalidOutput)?;
    Ok(scenario)
}
