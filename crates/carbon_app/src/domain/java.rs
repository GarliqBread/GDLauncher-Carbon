use anyhow::bail;
use regex::Regex;
use serde::{Deserialize, Serialize};

// TODO: This does not handle the case where the same path still exists between executions but changes the version
pub struct Java {
    pub component: JavaComponent,
    pub is_valid: bool,
}

impl From<crate::db::java::Data> for Java {
    fn from(value: crate::db::java::Data) -> Self {
        let is_valid = value.is_valid;
        Self {
            component: JavaComponent::from(value),
            is_valid,
        }
    }
}

pub enum JavaMajorVer {
    Version8,
    Version17,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct JavaComponent {
    pub path: String,
    pub arch: JavaArch,
    /// Indicates whether the component has manually been added by the user
    #[serde(rename = "type")]
    pub _type: JavaComponentType,
    pub version: JavaVersion,
}

impl From<crate::db::java::Data> for JavaComponent {
    fn from(value: crate::db::java::Data) -> Self {
        Self {
            path: value.path,
            arch: JavaArch::from(&*value.arch),
            _type: JavaComponentType::from(&*value.r#type),
            version: JavaVersion::try_from(&*value.full_version).unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum JavaComponentType {
    Local,
    Managed,
    Custom,
}

impl From<&str> for JavaComponentType {
    fn from(s: &str) -> Self {
        match &*s.to_lowercase() {
            "local" => Self::Local,
            "managed" => Self::Managed,
            _ => unreachable!("Uh oh, this shouldn't happen"),
        }
    }
}

impl From<JavaComponentType> for String {
    fn from(t: JavaComponentType) -> Self {
        match t {
            JavaComponentType::Local => "local",
            JavaComponentType::Managed => "managed",
            JavaComponentType::Custom => "custom",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum JavaArch {
    Amd64,
    X86,
    Aarch64,
}

impl<'a> From<JavaArch> for &'a str {
    fn from(arch: JavaArch) -> Self {
        match arch {
            JavaArch::Amd64 => "amd64",
            JavaArch::X86 => "x86",
            JavaArch::Aarch64 => "aarch64",
        }
    }
}

impl<'a> From<&'a str> for JavaArch {
    fn from(s: &'a str) -> Self {
        match s.to_lowercase().as_str() {
            "amd64" => JavaArch::Amd64,
            "x86" => JavaArch::X86,
            "aarch64" => JavaArch::Aarch64,
            _ => panic!("Unknown JavaArch: {s}"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct JavaVersion {
    pub major: u8,
    pub minor: Option<u8>,
    pub patch: Option<String>,
    pub update_number: Option<String>,
    pub prerelease: Option<String>,
    pub build_metadata: Option<String>,
}

impl TryFrom<&str> for JavaVersion {
    type Error = anyhow::Error;

    fn try_from(version_string: &str) -> Result<Self, Self::Error> {
        let regex = Regex::new(
            r#"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*(?:\.[0-9]+)?)(?:_(?P<update_number>[0-9]+)?)?(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<build_metadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?"#,
        )?;

        if let Some(captures) = regex.captures(version_string) {
            let mut version = JavaVersion {
                major: 0,
                minor: None,
                patch: None,
                update_number: None,
                prerelease: None,
                build_metadata: None,
            };

            for name in regex.capture_names().flatten() {
                match name {
                    "major" => {
                        if let Some(major) = captures.name("major") {
                            version.major = major.as_str().parse()?;
                        }
                    }
                    "minor" => {
                        if let Some(minor) = captures.name("minor") {
                            version.minor = Some(minor.as_str().parse()?);
                        }
                    }
                    "patch" => {
                        if let Some(patch) = captures.name("patch") {
                            version.patch = Some(patch.as_str().parse()?);
                        }
                    }
                    "update_number" => {
                        if let Some(update_number) = captures.name("update_number") {
                            version.update_number = Some(update_number.as_str().parse()?);
                        }
                    }
                    "prerelease" => {
                        if let Some(prerelease) = captures.name("prerelease") {
                            version.prerelease = Some(prerelease.as_str().to_string());
                        }
                    }
                    "build_metadata" => {
                        if let Some(build_metadata) = captures.name("build_metadata") {
                            version.build_metadata = Some(build_metadata.as_str().to_string());
                        }
                    }
                    _ => {
                        unreachable!("Regex capture group not handled: {}", name)
                    }
                }
            }
            return Ok(version);
        }

        bail!("Could not parse java version in string: {}", version_string);
    }
}

impl From<JavaVersion> for String {
    fn from(v: JavaVersion) -> Self {
        format!(
            "{}.{}.{}{}{}{}",
            v.major,
            v.minor.unwrap_or(0),
            v.patch.unwrap_or_default(),
            v.update_number.map(|u| format!("_{u}")).unwrap_or_default(),
            v.prerelease.map(|p| format!("-{p}")).unwrap_or_default(),
            v.build_metadata
                .map(|b| format!("+{b}"))
                .unwrap_or_default()
        )
    }
}

impl JavaVersion {
    pub fn from_major(major: u8) -> Self {
        Self {
            major,
            minor: None,
            patch: None,
            update_number: None,
            prerelease: None,
            build_metadata: None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::domain::java::JavaVersion;

    #[test]
    fn test_parse_java_version() {
        struct TestCase {
            output: &'static str,
            expected: Option<JavaVersion>,
        }

        let expected = [
            TestCase {
                output: "19.0.1",
                expected: Some(JavaVersion {
                    major: 19,
                    minor: Some(0),
                    patch: Some("1".to_owned()),
                    update_number: None,
                    prerelease: None,
                    build_metadata: None,
                }),
            },
            TestCase {
                output: "amd64",
                expected: None,
            },
            TestCase {
                output: "1.8.0_352-b08",
                expected: Some(JavaVersion {
                    major: 1,
                    minor: Some(8),
                    patch: Some("0".to_owned()),
                    update_number: Some("352".to_owned()),
                    prerelease: Some("b08".to_owned()),
                    build_metadata: None,
                }),
            },
            TestCase {
                output: "19.0.1+10",
                expected: Some(JavaVersion {
                    major: 19,
                    minor: Some(0),
                    patch: Some("1".to_owned()),
                    update_number: None,
                    prerelease: None,
                    build_metadata: Some("10".to_owned()),
                }),
            },
            TestCase {
                output: "1.4.0_03-b04",
                expected: Some(JavaVersion {
                    major: 1,
                    minor: Some(4),
                    patch: Some("0".to_owned()),
                    update_number: Some("03".to_owned()),
                    prerelease: Some("b04".to_owned()),
                    build_metadata: None,
                }),
            },
            TestCase {
                output: "17.0.6-beta+2-202211152348",
                expected: Some(JavaVersion {
                    major: 17,
                    minor: Some(0),
                    patch: Some("6".to_owned()),
                    update_number: None,
                    prerelease: Some("beta".to_owned()),
                    build_metadata: Some("2-202211152348".to_owned()),
                }),
            },
            TestCase {
                output: "1.8.0_362-beta-202211161809-b03+152",
                expected: Some(JavaVersion {
                    major: 1,
                    minor: Some(8),
                    patch: Some("0".to_owned()),
                    update_number: Some("362".to_owned()),
                    prerelease: Some("beta-202211161809-b03".to_owned()),
                    build_metadata: Some("152".to_owned()),
                }),
            },
            TestCase {
                output: "18.0.2.1+1",
                expected: Some(JavaVersion {
                    major: 18,
                    minor: Some(0),
                    patch: Some("2.1".to_owned()),
                    update_number: None,
                    prerelease: None,
                    build_metadata: Some("1".to_owned()),
                }),
            },
            TestCase {
                output: "17.0.5+8",
                expected: Some(JavaVersion {
                    major: 17,
                    minor: Some(0),
                    patch: Some("5".to_owned()),
                    update_number: None,
                    prerelease: None,
                    build_metadata: Some("8".to_owned()),
                }),
            },
            TestCase {
                output: "17.0.5+8-LTS",
                expected: Some(JavaVersion {
                    major: 17,
                    minor: Some(0),
                    patch: Some("5".to_owned()),
                    update_number: None,
                    prerelease: None,
                    build_metadata: Some("8-LTS".to_owned()),
                }),
            },
        ];

        for test_case in expected.iter() {
            let actual = JavaVersion::try_from(test_case.output).ok();
            assert_eq!(actual, test_case.expected);
        }
    }
}