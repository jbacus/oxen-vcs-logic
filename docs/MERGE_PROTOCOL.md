# Manual Merge Protocol for Logic Pro Projects

## Overview

Due to the binary nature of Logic Pro project files (.logicx), traditional line-based merge operations are not possible. This document outlines the **FCP XML Reconciliation Workflow** for manually merging divergent branches of Logic Pro projects.

## When Manual Merge is Required

Manual merge is necessary when:
1. Two users work on different branches of the same project
2. Both branches have made significant changes to tracks, regions, or automation
3. You want to combine changes from both branches into a unified version

## FCP XML Reconciliation Workflow

### Step 1: Export Both Versions to FCP XML

Logic Pro can export projects to Final Cut Pro XML format, which is a human-readable XML representation of the project structure.

**For Branch A (e.g., main branch):**
1. Open the project from branch A in Logic Pro
2. File → Export → Project to FCP XML
3. Save as `project_branch_a.xml`

**For Branch B (e.g., feature branch):**
1. Checkout branch B: `oxenvcs-cli checkout feature-branch`
2. Open the project in Logic Pro
3. File → Export → Project to FCP XML
4. Save as `project_branch_b.xml`

### Step 2: Manual Reconciliation

Since FCP XML is text-based, you can now:

1. **Compare the XML files** using a diff tool:
   ```bash
   diff project_branch_a.xml project_branch_b.xml
   ```

2. **Identify differences:**
   - New tracks added in each branch
   - Modified regions or automation
   - Plugin settings changes
   - Tempo/time signature changes

3. **Create a reconciled version:**
   - Copy unique tracks from both versions
   - Merge automation data manually
   - Resolve conflicting edits (keep one or the other)

### Step 3: Import Reconciled XML Back to Logic Pro

1. Create a new Logic Pro project (or start from one branch)
2. File → Import → FCP XML
3. Select your manually reconciled XML file
4. Logic Pro will reconstruct the project from the XML

### Step 4: Verify and Commit

1. Review the imported project thoroughly:
   - Check all tracks are present
   - Verify automation curves
   - Test plugin settings
   - Listen to the mix

2. Once verified, create a milestone commit:
   ```bash
   oxenvcs-cli commit --project /path/to/project.logicx \
     --message "Merge feature-branch into main via FCP XML reconciliation"
   ```

## Limitations

1. **Not all data is preserved in FCP XML:**
   - Some plugin-specific data may be lost
   - Certain Logic Pro-specific features (Flex Time, Drummer) may not export fully
   - Media files are referenced but not included

2. **Manual process:**
   - Requires human judgment to resolve conflicts
   - Time-consuming for large projects
   - Risk of human error

3. **Alternative for simple cases:**
   - If only a few tracks differ, consider copying individual tracks between projects manually in Logic Pro

## Best Practices

1. **Minimize divergence:**
   - Use file locking to prevent simultaneous edits
   - Communicate with team members before branching
   - Keep branches short-lived

2. **Document changes:**
   - Add clear commit messages
   - Use tags to mark important milestones
   - Keep a changelog of major structural changes

3. **Test before committing:**
   - Always open and test the merged project
   - Listen to the entire mix
   - Check for missing plugins or media

4. **Backup before merge:**
   - Keep both original branches intact
   - Create a separate merge branch
   - Don't delete source branches until merge is verified

## Future Improvements

When Oxen VCS adds native merge support for binary files, this manual process may be automated. Until then, FCP XML provides the most reliable way to merge Logic Pro projects.

## See Also

- [Apple Logic Pro XML Documentation](https://developer.apple.com/documentation/)
- [FCP XML Specification](https://developer.apple.com/library/archive/documentation/FinalCutProX/Reference/FinalCutProXXMLFormat/Introduction/Introduction.html)
- `docs/COLLABORATION.md` - Team workflow guidelines
