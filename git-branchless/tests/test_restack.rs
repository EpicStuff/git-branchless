use lib::testing::{GitInitOptions, GitRunOptions, make_git, remove_rebase_lines};

#[test]
fn test_restack_amended_commit() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }

    git.init_repo()?;

    git.detach_head()?;
    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;
    git.run(&["checkout", "HEAD^^"])?;
    git.run(&["commit", "--amend", "-m", "amend test1.txt"])?;

    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        O f777ecc (master) create initial.txt
        |\
        | @ 024c35c amend test1.txt
        |
        x 62fc20d (rewritten as 024c35ce) create test1.txt
        |
        o 96d1c37 create test2.txt
        |
        o 70deb1e create test3.txt
        hint: there is 1 abandoned commit in your commit graph
        hint: to fix this, run: git restack
        hint: disable this hint by running: git config --global branchless.hint.smartlogFixAbandoned false
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless("restack", &["--on-disk"])?;
        let stdout = remove_rebase_lines(stdout);
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        "###);
    }

    Ok(())
}

#[test]
fn test_restack_amended_commit_auto_print() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }

    git.init_repo()?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;

    git.detach_head()?;
    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;
    git.run(&["checkout", "HEAD^^"])?;
    git.run(&["commit", "--amend", "-m", "amend test1.txt"])?;

    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        O f777ecc (master) create initial.txt
        |\
        | @ 024c35c amend test1.txt
        |
        x 62fc20d (rewritten as 024c35ce) create test1.txt
        |
        o 96d1c37 create test2.txt
        |
        o 70deb1e create test3.txt
        hint: there is 1 abandoned commit in your commit graph
        hint: to fix this, run: git restack
        hint: disable this hint by running: git config --global branchless.hint.smartlogFixAbandoned false
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless("restack", &["--on-disk"])?;
        let stdout = remove_rebase_lines(stdout);
        insta::assert_snapshot!(stdout, @"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        O f777ecc (master) create initial.txt
        |
        @ 024c35c amend test1.txt
        |
        o 8cd7de6 create test2.txt
        |
        o b9a0491 create test3.txt
        ");
    }

    Ok(())
}

#[test]
fn test_restack_reparent() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }

    git.init_repo()?;

    git.detach_head()?;
    let test1_oid = git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;
    git.run(&["checkout", &test1_oid.to_string()])?;
    git.write_file_txt("test1", "test1 amended")?;
    git.run(&["commit", ".", "--amend", "-m", "amend test1.txt"])?;

    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r"
        O f777ecc (master) create initial.txt
        |\
        | @ 10d9b3e amend test1.txt
        |
        x 62fc20d (rewritten as 10d9b3ee) create test1.txt
        |
        o 96d1c37 create test2.txt
        |
        o 70deb1e create test3.txt
        hint: there is 1 abandoned commit in your commit graph
        hint: to fix this, run: git restack
        hint: disable this hint by running: git config --global branchless.hint.smartlogFixAbandoned false
        ");
    }

    {
        let (stdout, _stderr) = git.branchless("restack", &["--reparent"])?;
        let stdout = remove_rebase_lines(stdout);
        insta::assert_snapshot!(stdout, @r"
        Attempting rebase in-memory...
        [1/2] Committed as: 5acd93e create test2.txt
        [2/2] Committed as: 7a9000d create test3.txt
        branchless: processing 2 rewritten commits
        In-memory rebase succeeded.
        Finished restacking commits.
        No abandoned branches to restack.
        ");
    }

    // test2 should be identical as before, undoing the test1 amendment
    // this behavior is similar to `amend --reparent`
    {
        let (stdout, _stderr) = git.run(&["show", "--pretty=format:", "5acd93e"])?;
        insta::assert_snapshot!(stdout, @r"
        diff --git a/test1.txt b/test1.txt
        index 68589b4..7432a8f 100644
        --- a/test1.txt
        +++ b/test1.txt
        @@ -1 +1 @@
        -test1 amended
        \ No newline at end of file
        +test1 contents
        diff --git a/test2.txt b/test2.txt
        new file mode 100644
        index 0000000..4e512d2
        --- /dev/null
        +++ b/test2.txt
        @@ -0,0 +1 @@
        +test2 contents
        ");
    }

    Ok(())
}

#[test]
fn test_restack_reparent_auto_print() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }

    git.init_repo()?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;

    git.detach_head()?;
    let test1_oid = git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;
    git.run(&["checkout", &test1_oid.to_string()])?;
    git.write_file_txt("test1", "test1 amended")?;
    git.run(&["commit", ".", "--amend", "-m", "amend test1.txt"])?;

    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r"
        O f777ecc (master) create initial.txt
        |\
        | @ 10d9b3e amend test1.txt
        |
        x 62fc20d (rewritten as 10d9b3ee) create test1.txt
        |
        o 96d1c37 create test2.txt
        |
        o 70deb1e create test3.txt
        hint: there is 1 abandoned commit in your commit graph
        hint: to fix this, run: git restack
        hint: disable this hint by running: git config --global branchless.hint.smartlogFixAbandoned false
        ");
    }

    {
        let (stdout, _stderr) = git.branchless("restack", &["--reparent"])?;
        let stdout = remove_rebase_lines(stdout);
        insta::assert_snapshot!(stdout, @"
        Attempting rebase in-memory...
        [1/2] Committed as: 5acd93e create test2.txt
        [2/2] Committed as: 7a9000d create test3.txt
        branchless: processing 2 rewritten commits
        In-memory rebase succeeded.
        Finished restacking commits.
        No abandoned branches to restack.
        O f777ecc (master) create initial.txt
        |
        @ 10d9b3e amend test1.txt
        |
        o 5acd93e create test2.txt
        |
        o 7a9000d create test3.txt
        ");
    }

    // test2 should be identical as before, undoing the test1 amendment
    // this behavior is similar to `amend --reparent`
    {
        let (stdout, _stderr) = git.run(&["show", "--pretty=format:", "5acd93e"])?;
        insta::assert_snapshot!(stdout, @r"
        diff --git a/test1.txt b/test1.txt
        index 68589b4..7432a8f 100644
        --- a/test1.txt
        +++ b/test1.txt
        @@ -1 +1 @@
        -test1 amended
        \ No newline at end of file
        +test1 contents
        diff --git a/test2.txt b/test2.txt
        new file mode 100644
        index 0000000..4e512d2
        --- /dev/null
        +++ b/test2.txt
        @@ -0,0 +1 @@
        +test2 contents
        ");
    }

    Ok(())
}

#[test]
fn test_restack_consecutive_rewrites() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }

    git.init_repo()?;

    git.detach_head()?;
    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;
    git.run(&["checkout", "HEAD^^"])?;
    git.run(&["commit", "--amend", "-m", "amend test1.txt v1"])?;
    git.run(&["commit", "--amend", "-m", "amend test1.txt v2"])?;

    {
        let (stdout, _stderr) = git.branchless("restack", &["--on-disk"])?;
        let stdout = remove_rebase_lines(stdout);

        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        "###)
    }

    Ok(())
}

#[test]
fn test_restack_consecutive_rewrites_auto_print() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }

    git.init_repo()?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;

    git.detach_head()?;
    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;
    git.run(&["checkout", "HEAD^^"])?;
    git.run(&["commit", "--amend", "-m", "amend test1.txt v1"])?;
    git.run(&["commit", "--amend", "-m", "amend test1.txt v2"])?;

    {
        let (stdout, _stderr) = git.branchless("restack", &["--on-disk"])?;
        let stdout = remove_rebase_lines(stdout);

        insta::assert_snapshot!(stdout, @"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        O f777ecc (master) create initial.txt
        |
        @ 662b451 amend test1.txt v2
        |
        o 8e9bbde create test2.txt
        |
        o 9dc6dd0 create test3.txt
        ")
    }

    Ok(())
}

#[test]
fn test_move_abandoned_branch() -> eyre::Result<()> {
    let git = make_git()?;

    git.init_repo()?;

    git.commit_file("test1", 1)?;
    git.detach_head()?;
    git.run(&["commit", "--amend", "-m", "amend test1.txt v1"])?;
    git.run(&["commit", "--amend", "-m", "amend test1.txt v2"])?;

    {
        let (stdout, _stderr) = git.branchless("restack", &[])?;
        let stdout = remove_rebase_lines(stdout);
        insta::assert_snapshot!(stdout, @r###"
        No abandoned commits to restack.
        branchless: processing 1 update: branch master
        Finished restacking branches.
        "###);
    }

    Ok(())
}

#[test]
fn test_move_abandoned_branch_auto_print() -> eyre::Result<()> {
    let git = make_git()?;

    git.init_repo()?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;

    git.commit_file("test1", 1)?;
    git.detach_head()?;
    git.run(&["commit", "--amend", "-m", "amend test1.txt v1"])?;
    git.run(&["commit", "--amend", "-m", "amend test1.txt v2"])?;

    {
        let (stdout, _stderr) = git.branchless("restack", &[])?;
        let stdout = remove_rebase_lines(stdout);
        insta::assert_snapshot!(stdout, @"
        No abandoned commits to restack.
        branchless: processing 1 update: branch master
        Finished restacking branches.
        :
        @ 662b451 (master) amend test1.txt v2
        ");
    }

    Ok(())
}

#[test]
fn test_amended_initial_commit() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }

    git.init_repo()?;
    git.run(&["config", "branchless.restack.preserveTimestamps", "true"])?;

    git.commit_file("test1", 1)?;
    git.run(&["checkout", "HEAD^"])?;
    git.run(&["commit", "--amend", "-m", "new initial commit"])?;

    {
        // FIXME: there is no topological relationship between the new initial commit and `master`,
        // so they shouldn't be connected in the smartlog.
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        @ 9a9f929 new initial commit
        :
        O 62fc20d (master) create test1.txt
        "###);
    }

    {
        let (stdout, _stderr) =
            git.branchless("restack", &["--on-disk", "--force-rewrite"])?;
        let stdout = remove_rebase_lines(stdout);
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        "###);
    }

    Ok(())
}

#[test]
fn test_amended_initial_commit_auto_print() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }

    git.init_repo()?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;
    git.run(&["config", "branchless.restack.preserveTimestamps", "true"])?;

    git.commit_file("test1", 1)?;
    git.run(&["checkout", "HEAD^"])?;
    git.run(&["commit", "--amend", "-m", "new initial commit"])?;

    {
        // FIXME: there is no topological relationship between the new initial commit and `master`,
        // so they shouldn't be connected in the smartlog.
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        @ 9a9f929 new initial commit
        :
        O 62fc20d (master) create test1.txt
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless("restack", &["--on-disk", "--force-rewrite"])?;
        let stdout = remove_rebase_lines(stdout);
        insta::assert_snapshot!(stdout, @"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        @ 9a9f929 new initial commit
        |
        O 6d85943 (master) create test1.txt
        ");
    }

    Ok(())
}

/// The normal implicit revset must keep excluding abandoned public commits.
/// An explicit revset must never be expanded by --force-rewrite either.
#[test]
fn test_restack_default_and_explicit_draft_exclude_public_commits() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;

    let old_initial_oid = git.run(&["rev-parse", "HEAD"])?.0.trim().to_owned();
    let public_child_oid = git.commit_file("public-child", 1)?;
    git.run(&["checkout", &old_initial_oid])?;
    git.run(&["commit", "--amend", "-m", "new initial commit"])?;

    let (stdout, _) = git.branchless("restack", &[])?;
    assert!(stdout.contains("No abandoned commits to restack."));
    assert_eq!(
        git.run(&["rev-parse", "master"])?.0.trim(),
        public_child_oid.to_string()
    );

    let (stdout, _) = git.branchless("restack", &["--force-rewrite", "draft()"])?;
    assert!(stdout.contains("No abandoned commits to restack."));
    assert_eq!(
        git.run(&["rev-parse", "master"])?.0.trim(),
        public_child_oid.to_string()
    );
    Ok(())
}

#[test]
fn test_restack_amended_master() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }

    git.init_repo()?;

    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.detach_head()?;
    git.run(&["checkout", "HEAD^"])?;
    git.run(&["commit", "--amend", "-m", "amended test1"])?;

    {
        let (stdout, _stderr) =
            git.branchless("restack", &["--on-disk", "--force-rewrite", "all()"])?;
        let stdout = remove_rebase_lines(stdout);
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        "###);
    }

    Ok(())
}

#[test]
fn test_restack_amended_master_auto_print() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }

    git.init_repo()?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;

    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.detach_head()?;
    git.run(&["checkout", "HEAD^"])?;
    git.run(&["commit", "--amend", "-m", "amended test1"])?;

    {
        let (stdout, _stderr) =
            git.branchless("restack", &["--on-disk", "--force-rewrite", "all()"])?;
        let stdout = remove_rebase_lines(stdout);
        insta::assert_snapshot!(stdout, @"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        :
        @ ae94dc2 amended test1
        |
        O 51452b5 (master) create test2.txt
        ");
    }

    Ok(())
}

#[test]
fn test_restack_aborts_during_rebase_conflict() -> eyre::Result<()> {
    let git = make_git()?;

    git.init_repo()?;
    git.run(&["branch", "foo"])?;
    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.run(&["checkout", "HEAD^"])?;

    git.write_file_txt("test2", "conflicting test2 contents")?;
    git.run(&["add", "."])?;
    git.run(&["commit", "--amend", "-m", "amend test1 with test2 conflict"])?;

    {
        let (stdout, _stderr) = git.branchless_with_options(
            "restack",
            &["-f", "all()"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stdout, @r###"
        Attempting rebase in-memory...
        This operation would cause a merge conflict:
        - (1 conflicting file) 96d1c37 create test2.txt
        To resolve merge conflicts, retry this operation with the --merge option.
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless_with_options(
            "restack",
            &["--force-rewrite", "--merge", "all()"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        let stdout = remove_rebase_lines(stdout);

        insta::assert_snapshot!(stdout, @r###"
        Attempting rebase in-memory...
        Failed to merge in-memory, trying again on-disk...
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        CONFLICT (add/add): Merge conflict in test2.txt
        Error: Could not restack commits (exit code 1).
        You can resolve the error and try running `git restack` again.
        "###);
    }

    Ok(())
}

#[test]
fn test_restack_multiple_amended() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }
    git.init_repo()?;

    git.detach_head()?;
    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;
    git.commit_file("test4", 4)?;

    git.run(&["checkout", "HEAD~"])?;
    git.run(&["commit", "--amend", "-m", "test3 amended"])?;
    git.run(&["checkout", "HEAD~"])?;
    git.run(&["commit", "--amend", "-m", "test2 amended"])?;
    git.run(&["checkout", "HEAD~"])?;

    {
        let (stdout, _stderr) = git.branchless("restack", &["--on-disk"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        "###);
    }

    Ok(())
}

#[test]
fn test_restack_multiple_amended_auto_print() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_committer_date_is_author_date()? {
        return Ok(());
    }
    git.init_repo()?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;

    git.detach_head()?;
    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;
    git.commit_file("test4", 4)?;

    git.run(&["checkout", "HEAD~"])?;
    git.run(&["commit", "--amend", "-m", "test3 amended"])?;
    git.run(&["checkout", "HEAD~"])?;
    git.run(&["commit", "--amend", "-m", "test2 amended"])?;
    git.run(&["checkout", "HEAD~"])?;

    {
        let (stdout, _stderr) = git.branchless("restack", &["--on-disk"])?;
        insta::assert_snapshot!(stdout, @"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        O f777ecc (master) create initial.txt
        |
        @ 62fc20d create test1.txt
        |
        o 22f3928 test2 amended
        |
        o 2fdee37 test3 amended
        |
        o 772b150 create test4.txt
        ");
    }

    Ok(())
}

#[test]
fn test_restack_single_of_many_commits() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_reference_transactions()? {
        return Ok(());
    }

    git.init_repo()?;
    let test1_oid = git.commit_file("test1", 1)?;
    git.detach_head()?;
    let test2_oid = git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;
    git.run(&["checkout", &test1_oid.to_string()])?;
    let test4_oid = git.commit_file("test4", 4)?;
    git.commit_file("test5", 5)?;

    git.run(&["checkout", &test2_oid.to_string()])?;
    git.run(&["commit", "--amend", "-m", "updated test2"])?;

    git.run(&["checkout", &test4_oid.to_string()])?;
    git.run(&["commit", "--amend", "-m", "updated test4"])?;

    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        :
        O 62fc20d (master) create test1.txt
        |\
        | @ 3bd716d updated test4
        |\
        | o 7357d2b updated test2
        |\
        | x 96d1c37 (rewritten as 7357d2b7) create test2.txt
        | |
        | o 70deb1e create test3.txt
        |
        x bf0d52a (rewritten as 3bd716d5) create test4.txt
        |
        o 848121c create test5.txt
        hint: there are 2 abandoned commits in your commit graph
        hint: to fix this, run: git restack
        hint: disable this hint by running: git config --global branchless.hint.smartlogFixAbandoned false
        "###);
    }

    {
        let (stdout, stderr) = git.branchless("restack", &["--on-disk", &test2_oid.to_string()])?;
        insta::assert_snapshot!(stderr, @r###"
        branchless: processing 1 update: ref HEAD
        branchless: processing 1 update: ref HEAD
        Executing: git branchless hook-detect-empty-commit 70deb1e28791d8e7dd5a1f0c871a51b91282562f
        Executing: git branchless hook-register-extra-post-rewrite-hook
        branchless: processing 1 rewritten commit
        branchless: creating working copy snapshot
        branchless: running command: <git-executable> checkout 3bd716d57489779ab1daf446f80e66e90b56ead7 --
        Previous HEAD position was 944f78d create test3.txt
        branchless: processing 1 update: ref HEAD
        HEAD is now at 3bd716d updated test4
        branchless: processing checkout
        :
        O 62fc20d (master) create test1.txt
        |\
        | @ 3bd716d updated test4
        |\
        | o 7357d2b updated test2
        | |
        | o 944f78d create test3.txt
        |
        x bf0d52a (rewritten as 3bd716d5) create test4.txt
        |
        o 848121c create test5.txt
        hint: there is 1 abandoned commit in your commit graph
        hint: to fix this, run: git restack
        hint: disable this hint by running: git config --global branchless.hint.smartlogFixAbandoned false
        Successfully rebased and updated detached HEAD.
        "###);
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        "###);
    }

    Ok(())
}

#[test]
fn test_restack_single_of_many_commits_auto_print() -> eyre::Result<()> {
    let git = make_git()?;

    if !git.supports_reference_transactions()? {
        return Ok(());
    }

    git.init_repo()?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;
    let test1_oid = git.commit_file("test1", 1)?;
    git.detach_head()?;
    let test2_oid = git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;
    git.run(&["checkout", &test1_oid.to_string()])?;
    let test4_oid = git.commit_file("test4", 4)?;
    git.commit_file("test5", 5)?;

    git.run(&["checkout", &test2_oid.to_string()])?;
    git.run(&["commit", "--amend", "-m", "updated test2"])?;

    git.run(&["checkout", &test4_oid.to_string()])?;
    git.run(&["commit", "--amend", "-m", "updated test4"])?;

    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        :
        O 62fc20d (master) create test1.txt
        |\
        | @ 3bd716d updated test4
        |\
        | o 7357d2b updated test2
        |\
        | x 96d1c37 (rewritten as 7357d2b7) create test2.txt
        | |
        | o 70deb1e create test3.txt
        |
        x bf0d52a (rewritten as 3bd716d5) create test4.txt
        |
        o 848121c create test5.txt
        hint: there are 2 abandoned commits in your commit graph
        hint: to fix this, run: git restack
        hint: disable this hint by running: git config --global branchless.hint.smartlogFixAbandoned false
        "###);
    }

    {
        let (stdout, stderr) = git.branchless("restack", &["--on-disk", &test2_oid.to_string()])?;
        insta::assert_snapshot!(stderr, @r"
        branchless: processing 1 update: ref HEAD
        branchless: processing 1 update: ref HEAD
        Executing: git branchless hook-detect-empty-commit 70deb1e28791d8e7dd5a1f0c871a51b91282562f
        Executing: git branchless hook-register-extra-post-rewrite-hook
        branchless: processing 1 rewritten commit
        branchless: creating working copy snapshot
        branchless: running command: <git-executable> checkout 3bd716d57489779ab1daf446f80e66e90b56ead7 --
        Previous HEAD position was 944f78d create test3.txt
        branchless: processing 1 update: ref HEAD
        HEAD is now at 3bd716d updated test4
        branchless: processing checkout
        :
        O 62fc20d (master) create test1.txt
        |\
        | @ 3bd716d updated test4
        |\
        | o 7357d2b updated test2
        | |
        | o 944f78d create test3.txt
        |
        x bf0d52a (rewritten as 3bd716d5) create test4.txt
        |
        o 848121c create test5.txt
        hint: there is 1 abandoned commit in your commit graph
        hint: to fix this, run: git restack
        hint: disable this hint by running: git config --global branchless.hint.smartlogFixAbandoned false
        Successfully rebased and updated detached HEAD.
        ");
        insta::assert_snapshot!(stdout, @r"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Finished restacking commits.
        No abandoned branches to restack.
        :
        O 62fc20d (master) create test1.txt
        |\
        | @ 3bd716d updated test4
        |\
        | o 7357d2b updated test2
        | |
        | o 944f78d create test3.txt
        |
        x bf0d52a (rewritten as 3bd716d5) create test4.txt
        |
        o 848121c create test5.txt
        hint: there is 1 abandoned commit in your commit graph
        hint: to fix this, run: git restack
        hint: disable this hint by running: git config --global branchless.hint.smartlogFixAbandoned false
        ");
    }

    Ok(())
}

/// Regression test for: https://github.com/arxanas/git-branchless/issues/209
#[test]
fn test_restack_unobserved_commit() -> eyre::Result<()> {
    let git = make_git()?;

    git.init_repo_with_options(&GitInitOptions {
        run_branchless_init: false,
        ..Default::default()
    })?;

    git.commit_file("test1", 1)?;
    git.run(&["checkout", "-b", "foo"])?;
    git.commit_file("test2", 2)?;
    git.detach_head()?;
    git.commit_file("test3", 3)?;

    git.branchless("init", &[])?;
    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        :
        O 62fc20d (master) create test1.txt
        |
        o 96d1c37 (foo) create test2.txt
        |
        @ 70deb1e create test3.txt
        "###);
    }

    // Only abandon `test3` now that it's been observed by the DAG.
    git.run(&["checkout", "foo"])?;

    git.run(&["commit", "--amend", "-m", "Updated test2"])?;
    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        :
        O 62fc20d (master) create test1.txt
        |
        @ f4229de (> foo) Updated test2
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless("restack", &[])?;
        insta::assert_snapshot!(stdout, @r###"
        No abandoned commits to restack.
        No abandoned branches to restack.
        "###);
    }

    Ok(())
}

#[test]
fn test_restack_unobserved_commit_auto_print() -> eyre::Result<()> {
    let git = make_git()?;

    git.init_repo_with_options(&GitInitOptions {
        run_branchless_init: false,
        ..Default::default()
    })?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;

    git.commit_file("test1", 1)?;
    git.run(&["checkout", "-b", "foo"])?;
    git.commit_file("test2", 2)?;
    git.detach_head()?;
    git.commit_file("test3", 3)?;

    git.branchless("init", &[])?;
    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        :
        O 62fc20d (master) create test1.txt
        |
        o 96d1c37 (foo) create test2.txt
        |
        @ 70deb1e create test3.txt
        "###);
    }

    // Only abandon `test3` now that it's been observed by the DAG.
    git.run(&["checkout", "foo"])?;

    git.run(&["commit", "--amend", "-m", "Updated test2"])?;
    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        :
        O 62fc20d (master) create test1.txt
        |
        @ f4229de (> foo) Updated test2
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless("restack", &[])?;
        insta::assert_snapshot!(stdout, @"
        No abandoned commits to restack.
        No abandoned branches to restack.
        :
        O 62fc20d (master) create test1.txt
        |
        @ f4229de (> foo) Updated test2
        ");
    }

    Ok(())
}

/// Regression test for https://github.com/arxanas/git-branchless/issues/236
#[test]
fn test_restack_checked_out_branch() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;

    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.run(&["branch", "foo"])?;
    git.run(&["checkout", "HEAD^"])?;
    git.run(&["commit", "--amend", "-m", "test1 amended"])?;
    git.run(&["checkout", "foo"])?;

    {
        let (stdout, _stderr) = git.branchless("restack", &["-f", "all()"])?;
        insta::assert_snapshot!(stdout, @r###"
        Attempting rebase in-memory...
        [1/1] Committed as: 59e7581 create test2.txt
        branchless: processing 2 updates: branch foo, branch master
        branchless: processing 1 rewritten commit
        branchless: running command: <git-executable> checkout foo --
        In-memory rebase succeeded.
        Finished restacking commits.
        No abandoned branches to restack.
        "###);
    }

    Ok(())
}

#[test]
fn test_restack_checked_out_branch_auto_print() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;

    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.run(&["branch", "foo"])?;
    git.run(&["checkout", "HEAD^"])?;
    git.run(&["commit", "--amend", "-m", "test1 amended"])?;
    git.run(&["checkout", "foo"])?;

    {
        let (stdout, _stderr) = git.branchless("restack", &["-f", "all()"])?;
        insta::assert_snapshot!(stdout, @"
        Attempting rebase in-memory...
        [1/1] Committed as: 59e7581 create test2.txt
        branchless: processing 2 updates: branch foo, branch master
        branchless: processing 1 rewritten commit
        branchless: running command: <git-executable> checkout foo --
        In-memory rebase succeeded.
        Finished restacking commits.
        No abandoned branches to restack.
        :
        @ 59e7581 (> foo, master) create test2.txt
        ");
    }

    Ok(())
}

/// Regression test for https://github.com/arxanas/git-branchless/issues/325
#[test]
fn test_restack_non_observed_branch_commit() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;

    git.commit_file("test1", 1)?;
    let test2_oid = git.commit_file("test2", 2)?;
    git.run(&["branch", "foo"])?;
    git.run(&["checkout", "HEAD^"])?;
    git.clear_event_log()?;
    git.run(&["commit", "--amend", "-m", "test1 amended"])?;
    git.run(&["checkout", "foo"])?;

    {
        let (stdout, _stderr) = git.branchless("restack", &["-f", "all()"])?;
        insta::assert_snapshot!(stdout, @r###"
        Attempting rebase in-memory...
        [1/1] Committed as: 59e7581 create test2.txt
        branchless: processing 2 updates: branch foo, branch master
        branchless: processing 1 rewritten commit
        branchless: running command: <git-executable> checkout foo --
        In-memory rebase succeeded.
        Finished restacking commits.
        No abandoned branches to restack.
        "###);
    }

    git.run(&["reset", "--hard", &test2_oid.to_string()])?;
    {
        let (stdout, _stderr) = git.run(&["sl"])?;
        insta::assert_snapshot!(stdout, @r###"
        O f777ecc create initial.txt
        |\
        : x 62fc20d (rewritten as 14042005) create test1.txt
        : |
        : % 96d1c37 (rewritten as 59e75818) (> foo) create test2.txt
        :
        O 59e7581 (master) create test2.txt
        "###);
    }

    {
        // If we didn't enable `branchless.restack.preserveTimestamps`, then
        // this would have created another commit and moved `foo` to it. (Below,
        // it just moves `foo`.) It's not clear what the best behavior here is;
        // the user might not expect the restack operation to create duplicates
        // of all descendant commits just because a branch pointed to one of
        // those descendants.
        let (stdout, _stderr) = git.branchless("restack", &["-f"])?;
        insta::assert_snapshot!(stdout, @r###"
        No abandoned commits to restack.
        branchless: processing 1 update: branch foo
        Finished restacking branches.
        "###);
    }

    Ok(())
}

#[test]
fn test_restack_non_observed_branch_commit_auto_print() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;

    git.commit_file("test1", 1)?;
    let test2_oid = git.commit_file("test2", 2)?;
    git.run(&["branch", "foo"])?;
    git.run(&["checkout", "HEAD^"])?;
    git.clear_event_log()?;
    git.run(&["commit", "--amend", "-m", "test1 amended"])?;
    git.run(&["checkout", "foo"])?;

    {
        let (stdout, _stderr) = git.branchless("restack", &["-f", "all()"])?;
        insta::assert_snapshot!(stdout, @"
        Attempting rebase in-memory...
        [1/1] Committed as: 59e7581 create test2.txt
        branchless: processing 2 updates: branch foo, branch master
        branchless: processing 1 rewritten commit
        branchless: running command: <git-executable> checkout foo --
        In-memory rebase succeeded.
        Finished restacking commits.
        No abandoned branches to restack.
        :
        @ 59e7581 (> foo, master) create test2.txt
        ");
    }

    git.run(&["reset", "--hard", &test2_oid.to_string()])?;
    {
        let (stdout, _stderr) = git.run(&["sl"])?;
        insta::assert_snapshot!(stdout, @r###"
        O f777ecc create initial.txt
        |\
        : x 62fc20d (rewritten as 14042005) create test1.txt
        : |
        : % 96d1c37 (rewritten as 59e75818) (> foo) create test2.txt
        :
        O 59e7581 (master) create test2.txt
        "###);
    }

    {
        // If we didn't enable `branchless.restack.preserveTimestamps`, then
        // this would have created another commit and moved `foo` to it. (Below,
        // it just moves `foo`.) It's not clear what the best behavior here is;
        // the user might not expect the restack operation to create duplicates
        // of all descendant commits just because a branch pointed to one of
        // those descendants.
        let (stdout, _stderr) = git.branchless("restack", &["-f"])?;
        insta::assert_snapshot!(stdout, @"
        No abandoned commits to restack.
        branchless: processing 1 update: branch foo
        Finished restacking branches.
        :
        @ 59e7581 (> foo, master) create test2.txt
        ");
    }

    Ok(())
}

#[test]
fn test_restack_on_disk_preserves_distinct_timestamps() -> eyre::Result<()> {
    use std::collections::HashMap;

    let git = make_git()?;
    git.init_repo()?;
    git.detach_head()?;
    git.commit_file("parent", 1)?;
    git.write_file_txt("dated", "dated contents\n")?;
    git.run(&["add", "."])?;
    git.run_with_options(
        &["commit", "-m", "dated child"],
        &GitRunOptions {
            env: HashMap::from([
                ("GIT_AUTHOR_DATE".into(), "2001-02-03T04:05:06+05:30".into()),
                (
                    "GIT_COMMITTER_DATE".into(),
                    "2013-04-05T06:07:08-03:00".into(),
                ),
            ]),
            ..Default::default()
        },
    )?;
    let original_oid = git.run(&["rev-parse", "HEAD"])?.0.trim().to_string();
    let original_dates = git
        .run(&["show", "-s", "--format=%aI%n%cI", &original_oid])?
        .0;
    git.run(&["branch", "dated"])?;
    git.run(&["checkout", "HEAD^"])?;
    git.run(&["commit", "--amend", "-m", "rewritten parent"])?;

    let (stdout, _) = git.branchless("restack", &["--on-disk"])?;
    assert!(!stdout.contains("No abandoned branches to restack.\n@"));
    let rewritten_oid = git.run(&["rev-parse", "dated"])?.0.trim().to_string();
    let rewritten_dates = git
        .run(&["show", "-s", "--format=%aI%n%cI", &rewritten_oid])?
        .0;
    assert_ne!(original_oid, rewritten_oid);
    assert_eq!(rewritten_dates, original_dates);
    Ok(())
}

#[test]
fn test_restack_on_disk_preserves_merge_timestamps() -> eyre::Result<()> {
    use std::collections::HashMap;

    let git = make_git()?;
    git.init_repo()?;
    git.detach_head()?;
    let base_oid = git.commit_file("base", 1)?;
    git.run(&["branch", "side"])?;
    git.commit_file("mainline", 2)?;
    git.run(&["branch", "mainline"])?;
    git.run(&["checkout", "side"])?;
    git.commit_file("side", 3)?;
    git.run(&["checkout", "mainline"])?;
    git.run_with_options(
        &["merge", "--no-ff", "side", "-m", "dated merge"],
        &GitRunOptions {
            env: HashMap::from([
                ("GIT_AUTHOR_DATE".into(), "2002-03-04T05:06:07+04:00".into()),
                (
                    "GIT_COMMITTER_DATE".into(),
                    "2014-05-06T07:08:09-04:00".into(),
                ),
            ]),
            ..Default::default()
        },
    )?;
    let original_oid = git.run(&["rev-parse", "HEAD"])?.0.trim().to_string();
    let original_dates = git
        .run(&["show", "-s", "--format=%aI%n%cI", &original_oid])?
        .0;
    git.run(&["branch", "merged"])?;
    git.run(&["checkout", &base_oid.to_string()])?;
    git.run(&["commit", "--amend", "-m", "rewritten base"])?;
    git.branchless("restack", &["--on-disk"])?;

    let rewritten_oid = git.run(&["rev-parse", "merged"])?.0.trim().to_string();
    let rewritten_dates = git
        .run(&["show", "-s", "--format=%aI%n%cI", &rewritten_oid])?
        .0;
    let parents = git
        .run(&["rev-list", "--parents", "-n", "1", &rewritten_oid])?
        .0;
    assert_ne!(original_oid, rewritten_oid);
    assert_eq!(rewritten_dates, original_dates);
    assert_eq!(parents.split_whitespace().count(), 3);
    Ok(())
}

#[test]
fn test_restack_rerere_autoupdate_continues() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;
    git.run(&["config", "rerere.enabled", "true"])?;
    git.run(&["config", "rerere.autoupdate", "true"])?;
    git.detach_head()?;
    git.commit_file_with_contents("clash", 1, "base\n")?;
    git.commit_file_with_contents("clash", 2, "parent\n")?;
    let child_oid = git.commit_file_with_contents("clash", 3, "child\n")?;
    git.run(&["branch", "child"])?;
    git.run(&["checkout", "HEAD^"])?;
    git.write_file_txt("clash", "amended\n")?;
    git.run(&["commit", "-a", "--amend", "-m", "amended parent"])?;

    git.branchless_with_options(
        "restack",
        &["--on-disk"],
        &GitRunOptions {
            expected_exit_code: 1,
            ..Default::default()
        },
    )?;
    git.write_file_txt("clash", "resolved\n")?;
    git.run(&["add", "clash.txt"])?;
    git.run(&["rerere"])?;
    git.run(&["rebase", "--abort"])?;

    let (stdout, _) = git.branchless("restack", &["--on-disk"])?;
    assert!(stdout.contains("Rerere staged all conflicts; continuing rebase..."));
    let rewritten_oid = git.run(&["rev-parse", "child"])?.0.trim().to_string();
    assert_ne!(rewritten_oid, child_oid.to_string());
    assert!(!git.get_repo()?.is_rebase_underway()?);
    Ok(())
}

/// A Branchless on-disk rebase must remain one undo transaction even when
/// manually resumed after multiple conflicts in separate Git processes.
#[test]
fn test_restack_manual_continuations_keep_original_transaction() -> eyre::Result<()> {
    use lib::core::eventlog::{EventLogDb, EventTransactionId};

    let git = make_git()?;
    if !git.supports_reference_transactions()? {
        return Ok(());
    }

    git.init_repo()?;
    git.detach_head()?;
    git.write_file_txt("first", "base first\n")?;
    git.write_file_txt("second", "base second\n")?;
    git.run(&["add", "."])?;
    git.run(&["commit", "-m", "parent"])?;
    git.write_file_txt("first", "child first\n")?;
    git.run(&["commit", "-am", "first child"])?;
    git.write_file_txt("second", "child second\n")?;
    git.run(&["commit", "-am", "second child"])?;
    let original_stack = git.run(&["rev-parse", "HEAD"])?.0.trim().to_owned();
    git.run(&["branch", "stack"])?;

    git.run(&["checkout", "HEAD~2"])?;
    git.write_file_txt("first", "new parent first\n")?;
    git.write_file_txt("second", "new parent second\n")?;
    git.run(&["commit", "-a", "--amend", "-m", "amended parent"])?;

    let repo = git.get_repo()?;
    let conn = repo.get_db_conn()?;
    let event_log_db = EventLogDb::new(&conn)?;
    let events_before = event_log_db.get_events()?.len();

    git.branchless_with_options(
        "restack",
        &["--on-disk"],
        &GitRunOptions {
            expected_exit_code: 1,
            ..Default::default()
        },
    )?;

    let transaction_id_path = git
        .get_repo()?
        .get_rebase_state_dir_path()
        .join("branchless_transaction_id");
    let original_id: EventTransactionId = std::fs::read_to_string(&transaction_id_path)?
        .trim()
        .parse()?;
    assert_eq!(
        event_log_db.get_transaction_message(original_id)?,
        "restack"
    );
    // The test repo sets `branchless.restack.preserveTimestamps`, so a failed
    // timestamp hook must be rerun on `git rebase --continue`.
    assert!(
        transaction_id_path
            .with_file_name("reschedule-failed-exec")
            .exists()
    );

    git.write_file_txt("first", "resolved first\n")?;
    git.run(&["add", "first.txt"])?;
    git.run_with_options(
        &["rebase", "--continue"],
        &GitRunOptions {
            expected_exit_code: 1,
            ..Default::default()
        },
    )?;
    assert!(git.get_repo()?.get_index()?.has_conflicts());
    let continued_id: EventTransactionId = std::fs::read_to_string(&transaction_id_path)?
        .trim()
        .parse()?;
    assert_eq!(continued_id, original_id);

    git.write_file_txt("second", "resolved second\n")?;
    git.run(&["add", "second.txt"])?;
    // `git branchless wrap` passes its own transaction ID to the hooks, so it
    // must reuse the rebase's transaction rather than make a new one.
    git.branchless("wrap", &["--", "rebase", "--continue"])?;
    assert!(!git.get_repo()?.is_rebase_underway()?);
    assert!(!transaction_id_path.exists());

    let rewritten_stack = git.run(&["rev-parse", "stack"])?.0.trim().to_owned();
    assert_ne!(rewritten_stack, original_stack);

    // The commits and final branch move must all belong to the original
    // transaction, not to one transaction for each manual continuation.
    let new_events = event_log_db.get_events()?;
    let new_events = &new_events[events_before..];
    assert!(!new_events.is_empty());
    assert!(
        new_events
            .iter()
            .all(|event| event.get_event_tx_id() == original_id)
    );

    // A single undo should restore the complete original stack ref.
    git.branchless("undo", &["--yes"])?;
    let restored_stack = git.run(&["rev-parse", "stack"])?.0.trim().to_owned();
    assert_eq!(restored_stack, original_stack);
    Ok(())
}

#[test]
fn test_restack_auto_print_smartlog_opt_in() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;
    git.detach_head()?;
    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.run(&["checkout", "HEAD^"])?;
    git.run(&["commit", "--amend", "-m", "amended test1"])?;
    git.run(&["config", "branchless.smartlog.autoPrint", "true"])?;

    let (stdout, _) = git.branchless("restack", &[])?;
    assert!(stdout.contains("Finished restacking commits."));
    assert!(stdout.lines().any(|line| line.starts_with("@ ")));
    Ok(())
}
