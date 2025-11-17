# Auxin Hooks

This directory contains workflow automation hooks for your Auxin repository.

## Hook Types

### Pre-Commit Hooks (`pre-commit/`)

Run **before** creating a commit. Can abort the commit if validation fails.

Use cases:
- Validate metadata completeness (BPM, sample rate, key)
- Check file sizes
- Run linting or formatting
- Verify project structure
- Custom validation logic

**Exit code 0** = continue with commit
**Exit code non-zero** = abort commit

### Post-Commit Hooks (`post-commit/`)

Run **after** a successful commit. Cannot abort the commit.

Use cases:
- Send notifications (email, Slack, Discord)
- Create backups
- Trigger CI/CD pipelines
- Update external tracking systems
- Run custom scripts

## Creating Custom Hooks

1. Create a script in the appropriate directory
2. Make it executable (`chmod +x hook-name`)
3. Use any scripting language (bash, python, ruby, etc.)

### Available Environment Variables

All hooks receive these environment variables:

- `AUXIN_MESSAGE` - Commit message
- `AUXIN_BPM` - BPM value (if set)
- `AUXIN_SAMPLE_RATE` - Sample rate (if set)
- `AUXIN_KEY` - Key signature (if set)
- `AUXIN_TAGS` - Comma-separated tags
- `AUXIN_REPO_PATH` - Path to the repository

### Example Hook (bash)

```bash
#!/bin/bash
# pre-commit/check-bpm

if [ -z "$AUXIN_BPM" ]; then
    echo "ERROR: BPM is required for all commits"
    exit 1  # Abort commit
fi

if (( $(echo "$AUXIN_BPM < 60" | bc -l) )); then
    echo "WARNING: BPM seems unusually low ($AUXIN_BPM)"
fi

exit 0  # Continue with commit
```

### Example Hook (python)

```python
#!/usr/bin/env python3
# post-commit/notify-slack

import os
import json
import urllib.request

webhook_url = "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

message = {
    "text": f"New commit: {os.environ['AUXIN_MESSAGE']}",
    "fields": [
        {"title": "BPM", "value": os.environ.get('AUXIN_BPM', 'N/A')},
        {"title": "Key", "value": os.environ.get('AUXIN_KEY', 'N/A')}
    ]
}

req = urllib.request.Request(
    webhook_url,
    data=json.dumps(message).encode('utf-8'),
    headers={'Content-Type': 'application/json'}
)

urllib.request.urlopen(req)
```

## Built-in Hooks

Install built-in hooks with:

```bash
auxin hooks install <hook-name>
```

Available built-in hooks:
- `validate-metadata` (pre-commit) - Ensure BPM and sample rate are set
- `check-file-sizes` (pre-commit) - Warn about large files
- `notify` (post-commit) - Send local notifications
- `backup` (post-commit) - Create timestamped backups

## Hook Execution Order

Hooks run in **alphabetical order** by filename. Use prefixes to control order:

```
pre-commit/
├── 00-validate-metadata
├── 10-check-sizes
└── 20-custom-checks
```

## Debugging Hooks

- Test hooks manually: `./pre-commit/hook-name`
- Set environment variables for testing
- Use `set -x` in bash scripts for debug output
- Check exit codes: `echo $?`

## Best Practices

1. **Keep hooks fast** - They run on every commit
2. **Test hooks** before enabling in production
3. **Use descriptive names** - Include what the hook does
4. **Handle errors gracefully** - Provide helpful error messages
5. **Don't modify working directory** in pre-commit hooks
6. **Log hook activity** for debugging

## Disabling Hooks Temporarily

Rename the hook to start with a dot:

```bash
mv pre-commit/validate-metadata pre-commit/.validate-metadata
```

Or delete it:

```bash
rm pre-commit/validate-metadata
```

## Getting Help

```bash
# List all hooks
auxin hooks list

# List available built-in hooks
auxin hooks builtins

# Get help
auxin hooks --help
```
