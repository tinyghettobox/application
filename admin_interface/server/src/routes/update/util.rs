use std::cmp::Ordering;

pub type SemVer = (u32, u32, u32, Option<u32>);

// Helper function to parse a semver version string into a tuple
pub fn parse_version(version: &str) -> Option<SemVer> {
    let mut parts = version.splitn(2, '-');
    let version_part = parts.next()?;
    let rc_part = parts
        .next()
        .and_then(|s| s.strip_prefix("rc"))
        .and_then(|s| s.parse::<u32>().ok());

    let mut nums = version_part.split('.').map(|n| n.parse::<u32>());
    let major = nums.next()?.ok()?;
    let minor = nums.next().unwrap_or(Ok(0)).ok()?;
    let patch = nums.next().unwrap_or(Ok(0)).ok()?;

    Some((major, minor, patch, rc_part))
}

pub fn compare_version(version_a: &SemVer, version_b: &SemVer) -> Ordering {
    let (major_a, minor_a, patch_a, release_candidate_a) = version_a;
    let (major_b, minor_b, patch_b, release_candidate_b) = version_b;

    if major_a != major_b {
        return major_a.cmp(&major_b);
    }
    if minor_a != minor_b {
        return minor_a.cmp(&minor_b);
    }
    if patch_a != patch_b {
        return patch_a.cmp(&patch_b);
    }
    if let Some(rc_a) = release_candidate_a {
        if let Some(rc_b) = release_candidate_b {
            return rc_a.cmp(&rc_b);
        }
        return Ordering::Greater; // version_a has a release candidate, version_b does not
    } else if release_candidate_b.is_some() {
        return Ordering::Less; // version_b has a release candidate, version_a does not
    }
    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3, None)));
        assert_eq!(parse_version("1.2.3-rc28"), Some((1, 2, 3, Some(28))));
        assert_eq!(parse_version("1.2"), Some((1, 2, 0, None)));
        assert_eq!(parse_version("1.2-rc1"), Some((1, 2, 0, Some(1))));
        assert_eq!(parse_version("1"), Some((1, 0, 0, None)));
        assert_eq!(parse_version("1-rc1"), Some((1, 0, 0, Some(1))));
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn test_compare_version() {
        assert_eq!(
            compare_version(&(1, 2, 3, Some(2)), &(1, 2, 3, Some(2))),
            Ordering::Equal
        );
        assert_eq!(
            compare_version(&(1, 2, 3, None), &(1, 2, 3, None)),
            Ordering::Equal
        );

        assert_eq!(
            compare_version(&(1, 2, 3, Some(0)), &(1, 2, 3, None)),
            Ordering::Greater
        );
        assert_eq!(
            compare_version(&(1, 2, 3, Some(2)), &(1, 2, 3, Some(1))),
            Ordering::Greater
        );
        assert_eq!(
            compare_version(&(1, 2, 3, Some(2)), &(1, 2, 2, Some(3))),
            Ordering::Greater
        );
        assert_eq!(
            compare_version(&(1, 2, 3, Some(2)), &(1, 1, 4, Some(3))),
            Ordering::Greater
        );
        assert_eq!(
            compare_version(&(1, 2, 3, Some(2)), &(0, 3, 4, Some(3))),
            Ordering::Greater
        );

        assert_eq!(
            compare_version(&(1, 2, 3, None), &(1, 2, 3, Some(0))),
            Ordering::Less
        );
        assert_eq!(
            compare_version(&(1, 2, 3, Some(2)), &(1, 2, 3, Some(3))),
            Ordering::Less
        );
        assert_eq!(
            compare_version(&(1, 2, 3, Some(2)), &(1, 2, 4, Some(1))),
            Ordering::Less
        );
        assert_eq!(
            compare_version(&(1, 2, 3, Some(2)), &(1, 3, 2, Some(1))),
            Ordering::Less
        );
        assert_eq!(
            compare_version(&(1, 2, 3, Some(2)), &(2, 1, 2, Some(1))),
            Ordering::Less
        );
    }
}
