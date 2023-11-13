use radicle::git::Oid;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Stats {
    #[serde(rename = "filesChanged")]
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Changes {
    pub path: String,
}

#[derive(Debug, Deserialize)]
struct Diff {
    pub added: Vec<Changes>,
    pub deleted: Vec<Changes>,
    pub moved: Vec<Changes>,
    pub copied: Vec<Changes>,
    pub modified: Vec<Changes>,
    pub stats: Stats,
}

#[derive(Debug, Deserialize)]
struct Committer {
    pub name: String,
    pub email: String,
    pub time: usize,
}

#[derive(Debug, Deserialize)]
struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
struct Commit {
    pub id: Oid,
    pub author: Author,
    pub summary: String,
    pub description: String,
    pub parents: Vec<Oid>,
    pub committer: Committer,
}

#[derive(Debug, Deserialize)]
struct CommitInfo {
    pub commit: Commit,
    pub diff: Diff,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn will_deserialize_successfully() {
        let json = r#"
        {
          "commit": {
            "id": "b847ab34d220a65994b951ebc0fe657add2f0aac",
            "author": {
              "name": "Alice",
              "email": "alice@email.com"
            },
            "summary": "A change",
            "description": "This the description of the change",
            "parents": [
              "0e91a31e9ef030dbb008aba592d420008a9aa621"
            ],
            "committer": {
              "name": "Alice",
              "email": "alice@email.com",
              "time": 1698240701
            }
          },
          "diff": {
            "added": [],
            "deleted": [],
            "moved": [],
            "copied": [],
            "modified": [
              {
                "path": "radicle-cli/src/commands/rm.rs",
                "diff": {
                  "type": "plain",
                  "hunks": [],
                  "stats": {
                    "additions": 2,
                    "deletions": 2
                  },
                  "eof": "noneMissing"
                },
                "old": {
                  "oid": "9b214cc03bed6f3c11c79bc37d8b5ba5d8eb7d47",
                  "mode": "blob"
                },
                "new": {
                  "oid": "2675db04446b1d3fcf8f3350777e452c0dfa9c00",
                  "mode": "blob"
                }
              }
            ],
            "stats": {
              "filesChanged": 1,
              "insertions": 2,
              "deletions": 2
            }
          },
          "files": {
            "2675db04446b1d3fcf8f3350777e452c0dfa9c00": {
              "binary": false,
              "content": "use std::ffi::OsString;\n"
            },
            "9b214cc03bed6f3c11c79bc37d8b5ba5d8eb7d47": {
              "binary": false,
              "content": "use std::ffi::OsString;\n"
            }
          },
          "branches": []
        }
        "#;

        let commit_info: CommitInfo = serde_json::from_str(json).unwrap();

        assert_eq!(commit_info.commit.id, "b847ab34d220a65994b951ebc0fe657add2f0aac".parse().unwrap());
        assert_eq!(commit_info.commit.author.name, "Alice");
        assert_eq!(commit_info.commit.author.email, "alice@email.com");
        assert_eq!(commit_info.commit.committer.name, "Alice");
        assert_eq!(commit_info.commit.committer.email, "alice@email.com");
        assert_eq!(commit_info.commit.committer.time, 1698240701);
        assert_eq!(commit_info.commit.summary, "A change");
        assert_eq!(commit_info.commit.description, "This the description of the change");
        assert_eq!(commit_info.commit.parents[0], "0e91a31e9ef030dbb008aba592d420008a9aa621".parse().unwrap());

        assert!(commit_info.diff.added.is_empty());
        assert!(commit_info.diff.deleted.is_empty());
        assert!(commit_info.diff.moved.is_empty());
        assert!(commit_info.diff.copied.is_empty());
        assert_eq!(commit_info.diff.modified[0].path, "radicle-cli/src/commands/rm.rs");
    }
}