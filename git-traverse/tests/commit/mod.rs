mod ancestor {
    use crate::hex_to_id;
    use git_hash::{oid, ObjectId};
    use git_odb::{linked::Store, pack::FindExt};
    use git_traverse::commit;

    struct TraversalAssertion<'a> {
        init_script: &'a str,
        tips: &'a [&'a str],
        expected: &'a [&'a str],
        mode: commit::Parents,
        sorting: commit::Sorting,
    }

    impl<'a> TraversalAssertion<'a> {
        fn new(init_script: &'a str, tips: &'a [&'a str], expected: &'a [&'a str]) -> Self {
            TraversalAssertion {
                init_script,
                tips,
                expected,
                mode: Default::default(),
                sorting: Default::default(),
            }
        }

        fn with_mode(&mut self, mode: commit::Parents) -> &mut Self {
            self.mode = mode;
            self
        }

        fn with_sorting(&mut self, sorting: commit::Sorting) -> &mut Self {
            self.sorting = sorting;
            self
        }
    }

    impl TraversalAssertion<'_> {
        fn setup(&self) -> crate::Result<(Store, Vec<ObjectId>, Vec<ObjectId>)> {
            let dir = git_testtools::scripted_fixture_repo_read_only(self.init_script)?;
            let store = Store::at(dir.join(".git").join("objects"))?;
            let tips: Vec<_> = self.tips.iter().copied().map(hex_to_id).collect();
            let expected: Vec<ObjectId> = tips
                .clone()
                .into_iter()
                .chain(self.expected.iter().map(|hex_id| hex_to_id(hex_id)))
                .collect();
            Ok((store, tips, expected))
        }
        fn check_with_predicate(&mut self, predicate: impl FnMut(&oid) -> bool) -> crate::Result<()> {
            let (store, tips, expected) = self.setup()?;

            let oids: Result<Vec<_>, _> = commit::Ancestors::filtered(
                tips,
                commit::ancestors::State::default(),
                move |oid, buf| store.find_commit_iter(oid, buf).ok().map(|t| t.0),
                predicate,
            )
            .sorting(self.sorting)
            .mode(self.mode)
            .collect();

            assert_eq!(oids?, expected);
            Ok(())
        }

        fn check(&self) -> crate::Result {
            let (store, tips, expected) = self.setup()?;
            let oids: Result<Vec<_>, _> =
                commit::Ancestors::new(tips, commit::ancestors::State::default(), move |oid, buf| {
                    store.find_commit_iter(oid, buf).ok().map(|t| t.0)
                })
                .sorting(self.sorting)
                .mode(self.mode)
                .collect();
            assert_eq!(oids?, expected);
            Ok(())
        }
    }

    #[test]
    fn linear_history_no_branch() -> crate::Result {
        TraversalAssertion::new(
            "make_traversal_repo_for_commits.sh",
            &["9556057aee5abb06912922e9f26c46386a816822"],
            &[
                "17d78c64cef6c33a10a604573fd2c429e477fd63",
                "9902e3c3e8f0c569b4ab295ddf473e6de763e1e7",
                "134385f6d781b7e97062102c6a483440bfda2a03",
            ],
        )
        .check()
    }

    #[test]
    fn simple_branch_with_merge() -> crate::Result {
        TraversalAssertion::new(
            "make_traversal_repo_for_commits.sh",
            &["01ec18a3ebf2855708ad3c9d244306bc1fae3e9b"],
            &[
                "efd9a841189668f1bab5b8ebade9cd0a1b139a37",
                "ce2e8ffaa9608a26f7b21afc1db89cadb54fd353",
                "9556057aee5abb06912922e9f26c46386a816822",
                "9152eeee2328073cf23dcf8e90c949170b711659",
                "17d78c64cef6c33a10a604573fd2c429e477fd63",
                "9902e3c3e8f0c569b4ab295ddf473e6de763e1e7",
                "134385f6d781b7e97062102c6a483440bfda2a03",
            ],
        )
        .check()
    }

    #[test]
    fn simple_branch_first_parent_only() -> crate::Result {
        TraversalAssertion::new(
            "make_traversal_repo_for_commits.sh",
            &["01ec18a3ebf2855708ad3c9d244306bc1fae3e9b"],
            &[
                "efd9a841189668f1bab5b8ebade9cd0a1b139a37",
                "9556057aee5abb06912922e9f26c46386a816822",
                "17d78c64cef6c33a10a604573fd2c429e477fd63",
                "9902e3c3e8f0c569b4ab295ddf473e6de763e1e7",
                "134385f6d781b7e97062102c6a483440bfda2a03",
            ],
        )
        .with_mode(commit::Parents::First)
        .check()
    }

    #[test]
    fn multiple_tips() -> crate::Result {
        TraversalAssertion::new(
            "make_traversal_repo_for_commits.sh",
            &[
                "01ec18a3ebf2855708ad3c9d244306bc1fae3e9b",
                "9556057aee5abb06912922e9f26c46386a816822",
            ],
            &[
                "efd9a841189668f1bab5b8ebade9cd0a1b139a37",
                "ce2e8ffaa9608a26f7b21afc1db89cadb54fd353",
                "17d78c64cef6c33a10a604573fd2c429e477fd63",
                "9152eeee2328073cf23dcf8e90c949170b711659",
                "9902e3c3e8f0c569b4ab295ddf473e6de763e1e7",
                "134385f6d781b7e97062102c6a483440bfda2a03",
            ],
        )
        .check()
    }

    #[test]
    fn filtered_commit_does_not_block_ancestors_reachable_from_another_commit() -> crate::Result {
        // I don't see a use case for the predicate returning false for a commit but return true for
        // at least one of its ancestors, so this test is kind of dubious. But we do want
        // `Ancestors` to not eagerly blacklist all of a commit's ancestors when blacklisting that
        // one commit, and this test happens to check that.
        TraversalAssertion::new(
            "make_traversal_repo_for_commits.sh",
            &["01ec18a3ebf2855708ad3c9d244306bc1fae3e9b"],
            &[
                "efd9a841189668f1bab5b8ebade9cd0a1b139a37",
                "ce2e8ffaa9608a26f7b21afc1db89cadb54fd353",
                "9556057aee5abb06912922e9f26c46386a816822",
                "17d78c64cef6c33a10a604573fd2c429e477fd63",
                "9902e3c3e8f0c569b4ab295ddf473e6de763e1e7",
                "134385f6d781b7e97062102c6a483440bfda2a03",
            ],
        )
        .check_with_predicate(|id| id != hex_to_id("9152eeee2328073cf23dcf8e90c949170b711659"))
    }

    #[test]
    fn predicate_only_called_once_even_if_fork_point() -> crate::Result {
        // The `self.seen` check should come before the `self.predicate` check, as we don't know how
        // expensive calling `self.predicate` may be.
        let mut seen = false;
        TraversalAssertion::new(
            "make_traversal_repo_for_commits.sh",
            &["01ec18a3ebf2855708ad3c9d244306bc1fae3e9b"],
            &[
                "efd9a841189668f1bab5b8ebade9cd0a1b139a37",
                "ce2e8ffaa9608a26f7b21afc1db89cadb54fd353",
                "9152eeee2328073cf23dcf8e90c949170b711659",
            ],
        )
        .check_with_predicate(move |id| {
            if id == hex_to_id("9556057aee5abb06912922e9f26c46386a816822") {
                assert!(!seen);
                seen = true;
                false
            } else {
                true
            }
        })
    }

    #[test]
    fn graph_sorted_commits() -> crate::Result {
        TraversalAssertion::new(
            "make_traversal_repo_for_commits_with_dates.sh",
            &["288e509293165cb5630d08f4185bdf2445bf6170"],
            &[
                "9902e3c3e8f0c569b4ab295ddf473e6de763e1e7",
                "bcb05040a6925f2ff5e10d3ae1f9264f2e8c43ac",
                "134385f6d781b7e97062102c6a483440bfda2a03",
            ],
        )
        .check()
    }

    #[test]
    fn committer_date_sorted_commits() -> crate::Result {
        TraversalAssertion::new(
            "make_traversal_repo_for_commits_with_dates.sh",
            &["288e509293165cb5630d08f4185bdf2445bf6170"],
            &[
                "bcb05040a6925f2ff5e10d3ae1f9264f2e8c43ac",
                "9902e3c3e8f0c569b4ab295ddf473e6de763e1e7",
                "134385f6d781b7e97062102c6a483440bfda2a03",
            ],
        )
        .with_sorting(commit::Sorting::ByCommitterDate)
        .check()
    }

    #[test]
    fn committer_date_sorted_commits_parents_only() -> crate::Result {
        TraversalAssertion::new(
            "make_traversal_repo_for_commits_with_dates.sh",
            &["288e509293165cb5630d08f4185bdf2445bf6170"],
            &[
                "9902e3c3e8f0c569b4ab295ddf473e6de763e1e7",
                "134385f6d781b7e97062102c6a483440bfda2a03",
            ],
        )
        .with_sorting(commit::Sorting::ByCommitterDate)
        .with_mode(commit::Parents::First)
        .check()
    }
}
