use std::path::PathBuf;

use regex::Regex;
use thiserror::Error;

pub struct MavenCoordinates {
    group_id: String,
    artifact_id: String,
    version: String,
}

#[derive(Error, Debug)]
pub enum MavenError {
    #[error("invalid maven coordinates")]
    InvalidCoordinates,
}

/// Needs to be in the format of `group:artifact:version`
/// This is not the full maven specification but should be enough for our use case
fn is_maven_coordinates(maven_coordinates: &str) -> bool {
    Regex::new(r#"^[a-zA-Z0-9._-]+:[a-zA-Z0-9._-]+:[0-9]+\.[0-9]+(\.[0-9]+)?(-[a-zA-Z0-9._-]+)*(\.[a-zA-Z0-9._-]+)*$"#)
        .expect("failed to compile maven coordinates regex!!!")
        .is_match(maven_coordinates)
}

fn parse_maven_coordinates(maven_coordinates: &str) -> Result<MavenCoordinates, MavenError> {
    let mut split = maven_coordinates.split(':');
    let group_id = split.next().ok_or(MavenError::InvalidCoordinates)?;
    let artifact_id = split.next().ok_or(MavenError::InvalidCoordinates)?;
    let version = split.next().ok_or(MavenError::InvalidCoordinates)?;

    Ok(MavenCoordinates {
        group_id: group_id.to_string(),
        artifact_id: artifact_id.to_string(),
        version: version.to_string(),
    })
}

impl MavenCoordinates {
    /// Needs to be in the format of `group:artifact:version`
    pub fn try_from(coordinates: String) -> Result<Self, MavenError> {
        let coordinates = coordinates.trim();
        if coordinates.is_empty() || !is_maven_coordinates(coordinates) {
            return Err(MavenError::InvalidCoordinates);
        }

        parse_maven_coordinates(coordinates)
    }

    pub fn into_pathbuf(self) -> PathBuf {
        PathBuf::new()
            .join(self.group_id)
            .join(self.artifact_id)
            .join(self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_coordinates() {
        assert!(is_maven_coordinates("com.example:example:1.0.0"));
        assert!(is_maven_coordinates("com.example:example:1.0"));
        assert!(is_maven_coordinates(
            "com.example:example-something:1.0.final"
        ));
        assert!(is_maven_coordinates(
            "com.example:example-something:1.0.0.Final-beta.1"
        ));
        assert!(is_maven_coordinates(
            "com.example.example:example-example:1.0.0"
        ));
        assert!(is_maven_coordinates(
            "com.example.example:example-example:1.0.0-SNAPSHOT"
        ));
        assert!(is_maven_coordinates(
            "com.example.example:example-example:1.0.0-beta.1"
        ));
    }

    #[test]
    fn test_invalid_coordinates() {
        assert!(!is_maven_coordinates(""));
        assert!(!is_maven_coordinates("com.example:example:1"));
        assert!(!is_maven_coordinates("com.example:example"));
        assert!(!is_maven_coordinates("com.example:example:1.0.0:extra"));
    }

    #[test]
    fn test_parse_coordinates() {
        let coordinates = "com.example:example:1.0.0".to_string();
        let parsed_coordinates = parse_maven_coordinates(&coordinates).unwrap();
        assert_eq!(parsed_coordinates.group_id, "com.example");
        assert_eq!(parsed_coordinates.artifact_id, "example");
        assert_eq!(parsed_coordinates.version, "1.0.0");

        let coordinates = "com.example.example:example-example:1.0.0-SNAPSHOT".to_string();
        let parsed_coordinates = parse_maven_coordinates(&coordinates).unwrap();
        assert_eq!(parsed_coordinates.group_id, "com.example.example");
        assert_eq!(parsed_coordinates.artifact_id, "example-example");
        assert_eq!(parsed_coordinates.version, "1.0.0-SNAPSHOT");
    }

    #[test]
    fn test_try_from() {
        let coordinates = "com.example:example:1.0.0".to_string();
        let parsed_coordinates = MavenCoordinates::try_from(coordinates).unwrap();
        assert_eq!(parsed_coordinates.group_id, "com.example");
        assert_eq!(parsed_coordinates.artifact_id, "example");
        assert_eq!(parsed_coordinates.version, "1.0.0");

        let coordinates = "".to_string();
        assert!(MavenCoordinates::try_from(coordinates).is_err());

        let coordinates = "com.example.example:example-example:1.0.0.0:extra".to_string();
        assert!(MavenCoordinates::try_from(coordinates).is_err());
    }

    #[test]
    fn test_into_pathbuf() {
        let coordinates = "com.example:example:1.0.0".to_string();
        let parsed_coordinates = MavenCoordinates::try_from(coordinates).unwrap();
        let path = parsed_coordinates.into_pathbuf();
        assert_eq!(
            path,
            PathBuf::from("com.example").join("example").join("1.0.0")
        );
    }
}