use radicle::cob::issue::IssueCounts;
use radicle::cob::patch::PatchCounts;
use radicle::git::Oid;
use radicle::identity::Id;
use radicle::prelude::{BranchName, Did};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(remote = "PatchCounts")]
pub struct PatchCountsDef {
    pub open: usize,
    pub draft: usize,
    pub archived: usize,
    pub merged: usize,
}

#[derive(Deserialize)]
#[serde(remote = "IssueCounts")]
pub struct IssueCountsDef {
    pub open: usize,
    pub closed: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    pub description: String,
    pub default_branch: BranchName,
    pub delegates: Vec<Did>,
    pub head: Oid,
    #[serde(with = "PatchCountsDef")]
    pub patches: PatchCounts,
    #[serde(with = "IssueCountsDef")]
    pub issues: IssueCounts,
    pub id: Id,
    pub trackings: usize,
}

#[cfg(test)]
mod tests {
    use radicle::git::RefString;

    use super::*;

    #[test]
    fn will_deserialize_successfully() {
        let json = r#"
        {
          "name": "heartwood",
          "description": "Radicle Heartwood Protocol & Stack ❤️🪵",
          "defaultBranch": "master",
          "delegates": [
            "did:key:z6MksFqXN3Yhqk8pTJdUGLwATkRfQvwZXPqR2qMEhbS9wzpT"
          ],
          "head": "1d167581f2fa2e8ef0e36a268db1affef7418c64",
          "patches": {
            "open": 16,
            "draft": 4,
            "archived": 1,
            "merged": 50
          },
          "issues": {
            "open": 7,
            "closed": 1
          },
          "id": "rad:z3gqcJUoA1n9HaHKufZs5FCSGazv5",
          "trackings": 62
        }
        "#;

        let project: Project = serde_json::from_str(json).unwrap();

        assert_eq!(project.name, "heartwood");
        assert_eq!(project.description, "Radicle Heartwood Protocol & Stack ❤️🪵");
        assert_eq!(project.default_branch, RefString::try_from("master").unwrap());
        assert_eq!(
            project.delegates[0],
            "did:key:z6MksFqXN3Yhqk8pTJdUGLwATkRfQvwZXPqR2qMEhbS9wzpT".parse().unwrap()
        );
        assert_eq!(project.head, "1d167581f2fa2e8ef0e36a268db1affef7418c64".parse().unwrap());

        assert_eq!(project.patches.open, 16);
        assert_eq!(project.patches.draft, 4);
        assert_eq!(project.patches.archived, 1);
        assert_eq!(project.patches.merged, 50);

        assert_eq!(project.issues.open, 7);
        assert_eq!(project.issues.closed, 1);

        assert_eq!(project.id, "rad:z3gqcJUoA1n9HaHKufZs5FCSGazv5".parse().unwrap());
        assert_eq!(project.trackings, 62);
    }
}
