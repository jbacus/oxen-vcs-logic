use crate::liboxen_stub::model::LocalRepository;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
}

pub fn list(_repo: &LocalRepository) -> Result<Vec<Branch>> {
    // STUB: In real implementation, this would list branches
    Ok(vec![
        Branch {
            name: "main".to_string(),
        },
        Branch {
            name: "draft".to_string(),
        },
    ])
}

pub fn current_branch(_repo: &LocalRepository) -> Result<Branch> {
    // STUB: In real implementation, this would get current branch
    Ok(Branch {
        name: "draft".to_string(),
    })
}

pub fn create_from_head(_repo: &LocalRepository, branch_name: &str) -> Result<()> {
    // STUB: In real implementation, this would create a branch
    println!("[STUB] Would create branch: {}", branch_name);
    Ok(())
}

pub fn delete(_repo: &LocalRepository, branch_name: &str) -> Result<()> {
    // STUB: In real implementation, this would delete a branch
    println!("[STUB] Would delete branch: {}", branch_name);
    Ok(())
}
