use anyhow::anyhow;
use git2::{Oid, Repository};

pub fn get_file_contents_from_commit(
    working: &Repository,
    commit_oid: Oid,
    path: &str,
) -> anyhow::Result<String> {
    let commit = working.find_commit(commit_oid).map_err(|_| {
        anyhow!("Failed to find commit {}", commit_oid)
    })?;

    let tree = commit.tree()?;

    if let Ok(entry) = tree.get_path(path.as_ref()) {
        if let Ok(blob) = entry.to_object(working) {
            if let Some(content) = blob.as_blob() {
                let content_str = String::from_utf8_lossy(content.content());
                return Ok(content_str.to_string());
            }
        }
    }

    Err(anyhow!("Path {} not found in commit {}", path, commit_oid))
}

#[cfg(test)]
mod tests {
    use git2::{Oid, Repository};

    use crate::repository::get_file_contents_from_commit;
    use crate::test::create_test_repository;

    #[test]
    fn will_return_an_error_when_the_oid_does_not_exist() {
        let (temp_dir, _) = create_test_repository();
        let repository = Repository::open(temp_dir.path()).unwrap();

        let result = get_file_contents_from_commit(&repository, Oid::zero(), ".unknown/config.yaml");

        assert!(result.is_err());
        let error = result.unwrap_err();
        let root_cause = error.root_cause();
        assert_eq!(format!("{}", root_cause), format!("Failed to find commit {}", Oid::zero()));
    }

    #[test]
    fn will_return_the_contents_when_the_path_exists() {
        let (temp_dir, oid) = create_test_repository();
        let repository = Repository::open(temp_dir.path()).unwrap();

        let result = get_file_contents_from_commit(&repository, oid, ".concourse/config.yaml");

        assert!(result.is_ok());

        let content = result.unwrap();
        assert_eq!(content, "jobs:");
    }

    #[test]
    fn will_return_an_error_when_the_path_does_not_exist() {
        let (temp_dir, oid) = create_test_repository();
        let repository = Repository::open(temp_dir.path()).unwrap();

        let path = ".unknown/config.yaml";
        let result = get_file_contents_from_commit(&repository, oid, path);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let root_cause = error.root_cause();
        assert_eq!(format!("{}", root_cause), format!("Path {} not found in commit {}", path, oid));
    }
}