use crate::liboxen_stub::model::{Commit, LocalRepository, StagedData};
use crate::liboxen_stub::opts::AddOpts;
use anyhow::Result;

pub async fn add(repo: &LocalRepository, opts: &AddOpts) -> Result<()> {
    // STUB: In real implementation, this would stage files
    println!("[STUB] Would add files to: {}", repo.path.display());
    for path in &opts.paths {
        println!("[STUB]   - {}", path.display());
    }
    Ok(())
}

pub async fn commit(repo: &LocalRepository, message: &str) -> Result<Commit> {
    // STUB: In real implementation, this would create a commit
    println!("[STUB] Would create commit in: {}", repo.path.display());
    println!("[STUB] Message: {}", message);

    Ok(Commit {
        id: "stub_commit_id_12345".to_string(),
        message: message.to_string(),
        author: "stub_author".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn status(repo: &LocalRepository) -> Result<StagedData> {
    // STUB: In real implementation, this would get repository status
    println!("[STUB] Would get status for: {}", repo.path.display());
    Ok(StagedData::empty())
}

pub async fn checkout(repo: &LocalRepository, commit_id: &str) -> Result<()> {
    // STUB: In real implementation, this would checkout a commit
    println!(
        "[STUB] Would checkout {} in: {}",
        commit_id,
        repo.path.display()
    );
    Ok(())
}
