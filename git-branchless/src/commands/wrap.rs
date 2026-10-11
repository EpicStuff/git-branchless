//! Wrap a user-provided Git command, so that `git-branchless` can do special
//! processing.

use std::process::Command;
use std::time::SystemTime;

use eyre::Context;
use itertools::Itertools;

use lib::core::eventlog::{
    BRANCHLESS_TRANSACTION_ID_ENV_VAR, EventLogDb, EventTransactionId, get_rebase_transaction_id,
};
use lib::git::{GitRunInfo, Repo};
use lib::util::{ExitCode, EyreExitOr};

fn pass_through_git_command_inner(
    git_run_info: &GitRunInfo,
    args: &[&str],
    event_tx_id: Option<EventTransactionId>,
) -> EyreExitOr<()> {
    let GitRunInfo {
        path_to_git,
        working_directory,
        env,
    } = git_run_info;
    let mut command = Command::new(path_to_git);
    command.current_dir(working_directory);
    command.args(args);
    command.env_clear();
    command.envs(env.iter());
    if let Some(event_tx_id) = event_tx_id {
        command.env(BRANCHLESS_TRANSACTION_ID_ENV_VAR, event_tx_id.to_string());
    }
    let exit_status = command.status().wrap_err("Running Git command")?;
    let exit_code: isize = exit_status.code().unwrap_or(1).try_into()?;
    let exit_code = ExitCode(exit_code);
    if exit_code.is_success() {
        Ok(Ok(()))
    } else {
        Ok(Err(exit_code))
    }
}

fn pass_through_git_command<S: AsRef<str> + std::fmt::Debug>(
    git_run_info: &GitRunInfo,
    args: &[S],
    event_tx_id: Option<EventTransactionId>,
) -> EyreExitOr<()> {
    pass_through_git_command_inner(
        git_run_info,
        args.iter().map(AsRef::as_ref).collect_vec().as_slice(),
        event_tx_id,
    )
}

fn make_event_tx_id<S: AsRef<str> + std::fmt::Debug>(
    git_run_info: &GitRunInfo,
    args: &[S],
) -> eyre::Result<EventTransactionId> {
    let now = SystemTime::now();

    // Skip Git's global options (such as `-c name=value`) to find the
    // subcommand, following any `-C <path>` to the repository Git will use.
    let mut repo_dir = git_run_info.working_directory.clone();
    let mut args = args.iter().map(AsRef::as_ref);
    let subcommand = loop {
        match args.next() {
            Some("-C") => {
                if let Some(path) = args.next() {
                    repo_dir.push(path);
                }
            }
            Some("-c" | "--git-dir" | "--work-tree" | "--namespace" | "--config-env") => {
                args.next();
            }
            Some(arg) if arg.starts_with('-') => {}
            subcommand => break subcommand,
        }
    };
    let repo = Repo::from_dir(&repo_dir)?;

    // Continuing or aborting a Branchless-managed rebase should stay in the
    // rebase's transaction rather than starting a new one, since hooks prefer
    // the transaction ID passed in the environment.
    if subcommand == Some("rebase") {
        if let Some(event_tx_id) = get_rebase_transaction_id(&repo)? {
            return Ok(event_tx_id);
        }
    }
    let conn = repo.get_db_conn()?;
    let event_log_db = EventLogDb::new(&conn)?;
    let event_tx_id = event_log_db.make_transaction_id(now, subcommand.unwrap_or("wrap"))?;
    Ok(event_tx_id)
}

/// Run the provided Git command, but wrapped in an event transaction.
pub fn wrap<S: AsRef<str> + std::fmt::Debug>(
    git_run_info: &GitRunInfo,
    args: &[S],
) -> EyreExitOr<()> {
    // We may not be able to make an event transaction ID (such as if there is
    // no repository in the current directory). Ignore the error in that case.
    let event_tx_id = make_event_tx_id(git_run_info, args).ok();

    let exit_code = pass_through_git_command(git_run_info, args, event_tx_id)?;
    Ok(exit_code)
}
