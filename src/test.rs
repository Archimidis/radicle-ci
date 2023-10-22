use std::fs;

use git2::{Oid, Repository};
use tempfile::TempDir;

pub fn create_test_repository() -> (TempDir, Oid) {
    let folder = TempDir::new().unwrap();
    let path = folder.path();
    let repo = Repository::init(folder.path()).unwrap();

    fs::create_dir(path.join(".concourse")).unwrap();
    fs::write(path.join(".concourse").join("config.yaml"), "jobs:").unwrap();
    let mut index = repo.index().unwrap();
    index
        .add_all(["."], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();

    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();

    let sig_time = git2::Time::new(1673003014, 0);
    let sig = git2::Signature::new("Alice Liddell", "alice@radicle.xyz", &sig_time).unwrap();
    let oid = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Add concourse config folder\n",
        &tree,
        &[],
    )
        .unwrap();

    (folder, oid)
}
