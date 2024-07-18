
#[derive(Debug, Clone, Ord)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub pre: Option<String>,
    pub build: Option<String>,
}

impl Version {
    pub fn new(major: u8, minor: u8, patch: u8, pre: Option<String>, build: Option<String>) -> Self {
        Self {
            major,
            minor,
            patch,
            pre,
            build,
        }
    }

    pub fn to_string(&self) -> String {
        let mut version = format!("v{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(pre) = &self.pre {
            version.push_str(&format!("-{}", pre));
        }
        if let Some(build) = &self.build {
            version.push_str(&format!("+{}", build));
        }
        version
    }

    pub fn as_str(&self) -> &str {
        self.to_string().as_str()
    }

    pub fn as_bytes(&self) -> &[u8] {
      let mut bytes = vec![self.major, self.minor, self.patch];

      if let Some(pre) = &self.pre {
        bytes.extend_from_slice(pre.as_bytes());
      }

      if let Some(build) = &self.build {
        bytes.extend_from_slice(build.as_bytes());
      }

      bytes.as_slice()
    }

    pub fn is_compatible(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor
    }

    pub fn is_stable(&self) -> bool {
        self.pre.is_none()
    }

    pub fn is_prerelease(&self) -> bool {
        self.pre.is_some()
    }

    pub fn is_build(&self) -> bool {
        self.build.is_some()
    }

    pub fn is_dev(&self) -> bool {
        self.is_prerelease() && self.pre.as_ref().unwrap() == "dev"
    }

    pub fn is_alpha(&self) -> bool {
        self.is_prerelease() && self.pre.as_ref().unwrap() == "alpha"
    }

    pub fn is_beta(&self) -> bool {
        self.is_prerelease() && self.pre.as_ref().unwrap() == "beta"
    }

    pub fn is_rc(&self) -> bool {
        self.is_prerelease() && self.pre.as_ref().unwrap() == "rc"
    }

    pub fn is_nightly(&self) -> bool {
        self.is_build() && self.build.as_ref().unwrap() == "nightly"
    }

    pub fn is_ci(&self) -> bool {
        self.is_build() && self.build.as_ref().unwrap() == "ci"
    }
}

impl From<&str> for Version {
    fn from(version: &str) -> Self {
        let mut version = crate::unwrap!(version.strip_prefix("v")).splitn(2, '-');
        let version = version.next().unwrap();
        let mut version = version.splitn(2, '+');
        let version = version.next().unwrap();
        let mut version = version.splitn(3, '.');
        let major = version.next().unwrap().parse().unwrap();
        let minor = version.next().unwrap().parse().unwrap();
        let mut version = version.next().unwrap().splitn(2, '.');
        let patch = version.next().unwrap().parse().unwrap();
        let pre = version.next().map(|s| s.to_string());
        let build = None;
        Self::new(major, minor, patch, pre, build)
    }
}

impl From<Version> for String {
    fn from(version: Version) -> Self {
        let mut v = format!("v{}.{}.{}", version.major, version.minor, version.patch);
        if let Some(pre) = version.pre {
            v.push_str(&format!("-{}", pre));
        }
        if let Some(build) = version.build {
            v.push_str(&format!("+{}", build));
        }
        version
    }
}

impl From<&[u8]> for Version {
    fn from(bytes: &[u8]) -> Self {
        let mut bytes = bytes.to_vec();
        let major = bytes.remove(0);
        let minor = bytes.remove(0);
        let patch = bytes.remove(0);
        let mut pre = None;
        let mut build = None;
        if let Some(index) = bytes.iter().position(|&b| b == b'-') {
            pre = Some(String::from_utf8(bytes.drain(index..).collect()).unwrap());
        }
        if let Some(index) = bytes.iter().position(|&b| b == b'+') {
            build = Some(String::from_utf8(bytes.drain(index..).collect()).unwrap());
        }
        Self::new(major, minor, patch, pre, build)
    }
}

impl From<Version> for &[u8] {
    fn from(version: Version) -> Self {
        let bytes = vec![version.major.to_le_bytes(), version.minor.to_le_bytes(), version.patch.to_le_bytes()];
        let mut bytes = bytes.concat();

        if let Some(pre) = version.pre {
          bytes.extend_from_slice(pre.as_bytes());
        }

        if let Some(build) = version.build {
          bytes.extend_from_slice(build.as_bytes());
        }

        bytes.as_slice()
    }
}

impl From<Vec<u8>> for Version {
    fn from(bytes: Vec<u8>) -> Self {
        Self::from(bytes.as_slice())
    }
}

impl From<Version> for Vec<u8> {
    fn from(version: Version) -> Self {
        Self::from(version.as_bytes())
    }
}

impl From<(u8, u8, u8)> for Version {
    fn from((major, minor, patch): (u8, u8, u8)) -> Self {
        Self::new(major, minor, patch, None, None)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.pre == other.pre
            && self.build == other.build
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.major != other.major {
            return Some(self.major.cmp(&other.major));
        }
        if self.minor != other.minor {
            return Some(self.minor.cmp(&other.minor));
        }
        if self.patch != other.patch {
            return Some(self.patch.cmp(&other.patch));
        }
        if self.pre.is_some() && other.pre.is_none() {
            return Some(std::cmp::Ordering::Less);
        }
        if self.pre.is_none() && other.pre.is_some() {
            return Some(std::cmp::Ordering::Greater);
        }
        if self.pre.is_some() && other.pre.is_some() {
            if self.pre != other.pre {
                return Some(self.pre.cmp(&other.pre));
            }
        }
        Some(std::cmp::Ordering::Equal)
    }
}
