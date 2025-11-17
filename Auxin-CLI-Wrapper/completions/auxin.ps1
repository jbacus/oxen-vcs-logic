
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'auxin' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'auxin'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'auxin' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize a new Oxen repository for a Logic Pro project')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Stage changes to be committed')
            [CompletionResult]::new('commit', 'commit', [CompletionResultType]::ParameterValue, 'Create a commit with optional audio metadata')
            [CompletionResult]::new('log', 'log', [CompletionResultType]::ParameterValue, 'Show commit history')
            [CompletionResult]::new('restore', 'restore', [CompletionResultType]::ParameterValue, 'Restore project to a previous commit')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show repository status')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show detailed information about a commit')
            [CompletionResult]::new('diff', 'diff', [CompletionResultType]::ParameterValue, 'Show changes between commits or working directory')
            [CompletionResult]::new('compare', 'compare', [CompletionResultType]::ParameterValue, 'Compare metadata between two commits')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search commit history with advanced filtering')
            [CompletionResult]::new('lock', 'lock', [CompletionResultType]::ParameterValue, 'Manage project locks for team collaboration')
            [CompletionResult]::new('auth', 'auth', [CompletionResultType]::ParameterValue, 'Authenticate with Oxen Hub for remote collaboration')
            [CompletionResult]::new('metadata-diff', 'metadata-diff', [CompletionResultType]::ParameterValue, 'Compare metadata between two Logic Pro project versions')
            [CompletionResult]::new('daemon', 'daemon', [CompletionResultType]::ParameterValue, 'Control the background daemon service')
            [CompletionResult]::new('hooks', 'hooks', [CompletionResultType]::ParameterValue, 'Manage workflow automation hooks')
            [CompletionResult]::new('console', 'console', [CompletionResultType]::ParameterValue, 'Launch interactive console for real-time monitoring')
            [CompletionResult]::new('activity', 'activity', [CompletionResultType]::ParameterValue, 'Show recent project activity timeline')
            [CompletionResult]::new('team', 'team', [CompletionResultType]::ParameterValue, 'Show team members and their contributions')
            [CompletionResult]::new('queue', 'queue', [CompletionResultType]::ParameterValue, 'Manage offline operation queue')
            [CompletionResult]::new('comment', 'comment', [CompletionResultType]::ParameterValue, 'Manage comments on commits')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;init' {
            [CompletionResult]::new('--logic', '--logic', [CompletionResultType]::ParameterName, 'Initialize for Logic Pro project (auto-detect and configure)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;add' {
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Stage all changes in the repository')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Stage all changes in the repository')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;commit' {
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'Commit message describing the changes')
            [CompletionResult]::new('--message', '--message', [CompletionResultType]::ParameterName, 'Commit message describing the changes')
            [CompletionResult]::new('--bpm', '--bpm', [CompletionResultType]::ParameterName, 'Beats per minute (tempo) of the project')
            [CompletionResult]::new('--sample-rate', '--sample-rate', [CompletionResultType]::ParameterName, 'Sample rate in Hz (e.g., 44100, 48000, 96000)')
            [CompletionResult]::new('--key', '--key', [CompletionResultType]::ParameterName, 'Key signature (e.g., ''C Major'', ''A Minor'', ''F# Minor'')')
            [CompletionResult]::new('--tags', '--tags', [CompletionResultType]::ParameterName, 'Tags for categorization (comma-separated, e.g., ''mixing,draft'')')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;log' {
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Maximum number of commits to display')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'Maximum number of commits to display')
            [CompletionResult]::new('--bpm', '--bpm', [CompletionResultType]::ParameterName, 'Filter by BPM (e.g., 120, 128)')
            [CompletionResult]::new('--tag', '--tag', [CompletionResultType]::ParameterName, 'Filter by tag (e.g., ''mixing'', ''vocals'')')
            [CompletionResult]::new('--key', '--key', [CompletionResultType]::ParameterName, 'Filter by key signature (e.g., ''C Major'')')
            [CompletionResult]::new('--since', '--since', [CompletionResultType]::ParameterName, 'Show commits since date (YYYY-MM-DD)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;restore' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;status' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;show' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;diff' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;compare' {
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format (text, colored, json, compact)')
            [CompletionResult]::new('--plain', '--plain', [CompletionResultType]::ParameterName, 'Disable colored output')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;search' {
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format (list, compact, json)')
            [CompletionResult]::new('--ranked', '--ranked', [CompletionResultType]::ParameterName, 'Sort results by relevance score')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;lock' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('acquire', 'acquire', [CompletionResultType]::ParameterValue, 'Acquire exclusive lock for editing')
            [CompletionResult]::new('release', 'release', [CompletionResultType]::ParameterValue, 'Release the lock you currently hold')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current lock status')
            [CompletionResult]::new('break', 'break', [CompletionResultType]::ParameterValue, 'Force break an existing lock (admin only)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;lock;acquire' {
            [CompletionResult]::new('--timeout', '--timeout', [CompletionResultType]::ParameterName, 'Lock timeout in hours')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;lock;release' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;lock;status' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;lock;break' {
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Confirm you want to force break the lock')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;lock;help' {
            [CompletionResult]::new('acquire', 'acquire', [CompletionResultType]::ParameterValue, 'Acquire exclusive lock for editing')
            [CompletionResult]::new('release', 'release', [CompletionResultType]::ParameterValue, 'Release the lock you currently hold')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current lock status')
            [CompletionResult]::new('break', 'break', [CompletionResultType]::ParameterValue, 'Force break an existing lock (admin only)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;lock;help;acquire' {
            break
        }
        'auxin;lock;help;release' {
            break
        }
        'auxin;lock;help;status' {
            break
        }
        'auxin;lock;help;break' {
            break
        }
        'auxin;lock;help;help' {
            break
        }
        'auxin;auth' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('login', 'login', [CompletionResultType]::ParameterValue, 'Login to Oxen Hub with API credentials')
            [CompletionResult]::new('logout', 'logout', [CompletionResultType]::ParameterValue, 'Logout from Oxen Hub')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current authentication status')
            [CompletionResult]::new('test', 'test', [CompletionResultType]::ParameterValue, 'Test authentication with Oxen Hub')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;auth;login' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;auth;logout' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;auth;status' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;auth;test' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;auth;help' {
            [CompletionResult]::new('login', 'login', [CompletionResultType]::ParameterValue, 'Login to Oxen Hub with API credentials')
            [CompletionResult]::new('logout', 'logout', [CompletionResultType]::ParameterValue, 'Logout from Oxen Hub')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current authentication status')
            [CompletionResult]::new('test', 'test', [CompletionResultType]::ParameterValue, 'Test authentication with Oxen Hub')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;auth;help;login' {
            break
        }
        'auxin;auth;help;logout' {
            break
        }
        'auxin;auth;help;status' {
            break
        }
        'auxin;auth;help;test' {
            break
        }
        'auxin;auth;help;help' {
            break
        }
        'auxin;metadata-diff' {
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Output format (text or json)')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'Use colored output (default: auto-detect)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Include technical details in output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Include technical details in output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;daemon' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Check daemon status')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start the daemon service')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop the daemon service')
            [CompletionResult]::new('restart', 'restart', [CompletionResultType]::ParameterValue, 'Restart the daemon service')
            [CompletionResult]::new('logs', 'logs', [CompletionResultType]::ParameterValue, 'Show daemon logs')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;daemon;status' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;daemon;start' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;daemon;stop' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;daemon;restart' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;daemon;logs' {
            [CompletionResult]::new('--lines', '--lines', [CompletionResultType]::ParameterName, 'Number of log lines to show')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;daemon;help' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Check daemon status')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start the daemon service')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop the daemon service')
            [CompletionResult]::new('restart', 'restart', [CompletionResultType]::ParameterValue, 'Restart the daemon service')
            [CompletionResult]::new('logs', 'logs', [CompletionResultType]::ParameterValue, 'Show daemon logs')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;daemon;help;status' {
            break
        }
        'auxin;daemon;help;start' {
            break
        }
        'auxin;daemon;help;stop' {
            break
        }
        'auxin;daemon;help;restart' {
            break
        }
        'auxin;daemon;help;logs' {
            break
        }
        'auxin;daemon;help;help' {
            break
        }
        'auxin;hooks' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize hooks directory')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all installed hooks')
            [CompletionResult]::new('builtins', 'builtins', [CompletionResultType]::ParameterValue, 'List available built-in hooks')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Install a built-in hook')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove an installed hook')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;hooks;init' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'auxin;hooks;list' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'auxin;hooks;builtins' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'auxin;hooks;install' {
            [CompletionResult]::new('--hook-type', '--hook-type', [CompletionResultType]::ParameterName, 'Hook type (pre-commit or post-commit)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'auxin;hooks;remove' {
            [CompletionResult]::new('--hook-type', '--hook-type', [CompletionResultType]::ParameterName, 'Hook type (pre-commit or post-commit)')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'auxin;hooks;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize hooks directory')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all installed hooks')
            [CompletionResult]::new('builtins', 'builtins', [CompletionResultType]::ParameterValue, 'List available built-in hooks')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Install a built-in hook')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove an installed hook')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;hooks;help;init' {
            break
        }
        'auxin;hooks;help;list' {
            break
        }
        'auxin;hooks;help;builtins' {
            break
        }
        'auxin;hooks;help;install' {
            break
        }
        'auxin;hooks;help;remove' {
            break
        }
        'auxin;hooks;help;help' {
            break
        }
        'auxin;console' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;activity' {
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'Number of activities to show')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;team' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;queue' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show pending operations in the queue')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Manually sync all pending operations')
            [CompletionResult]::new('clear', 'clear', [CompletionResultType]::ParameterValue, 'Clear completed operations from the queue')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a specific operation from the queue')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;queue;status' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;queue;sync' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;queue;clear' {
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Clear all operations including pending')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;queue;remove' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;queue;help' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show pending operations in the queue')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Manually sync all pending operations')
            [CompletionResult]::new('clear', 'clear', [CompletionResultType]::ParameterValue, 'Clear completed operations from the queue')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a specific operation from the queue')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;queue;help;status' {
            break
        }
        'auxin;queue;help;sync' {
            break
        }
        'auxin;queue;help;clear' {
            break
        }
        'auxin;queue;help;remove' {
            break
        }
        'auxin;queue;help;help' {
            break
        }
        'auxin;comment' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a comment to a commit')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List comments on a commit')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;comment;add' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;comment;list' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;comment;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a comment to a commit')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List comments on a commit')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;comment;help;add' {
            break
        }
        'auxin;comment;help;list' {
            break
        }
        'auxin;comment;help;help' {
            break
        }
        'auxin;completions' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Enable verbose debug output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'auxin;help' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize a new Oxen repository for a Logic Pro project')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Stage changes to be committed')
            [CompletionResult]::new('commit', 'commit', [CompletionResultType]::ParameterValue, 'Create a commit with optional audio metadata')
            [CompletionResult]::new('log', 'log', [CompletionResultType]::ParameterValue, 'Show commit history')
            [CompletionResult]::new('restore', 'restore', [CompletionResultType]::ParameterValue, 'Restore project to a previous commit')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show repository status')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show detailed information about a commit')
            [CompletionResult]::new('diff', 'diff', [CompletionResultType]::ParameterValue, 'Show changes between commits or working directory')
            [CompletionResult]::new('compare', 'compare', [CompletionResultType]::ParameterValue, 'Compare metadata between two commits')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search commit history with advanced filtering')
            [CompletionResult]::new('lock', 'lock', [CompletionResultType]::ParameterValue, 'Manage project locks for team collaboration')
            [CompletionResult]::new('auth', 'auth', [CompletionResultType]::ParameterValue, 'Authenticate with Oxen Hub for remote collaboration')
            [CompletionResult]::new('metadata-diff', 'metadata-diff', [CompletionResultType]::ParameterValue, 'Compare metadata between two Logic Pro project versions')
            [CompletionResult]::new('daemon', 'daemon', [CompletionResultType]::ParameterValue, 'Control the background daemon service')
            [CompletionResult]::new('hooks', 'hooks', [CompletionResultType]::ParameterValue, 'Manage workflow automation hooks')
            [CompletionResult]::new('console', 'console', [CompletionResultType]::ParameterValue, 'Launch interactive console for real-time monitoring')
            [CompletionResult]::new('activity', 'activity', [CompletionResultType]::ParameterValue, 'Show recent project activity timeline')
            [CompletionResult]::new('team', 'team', [CompletionResultType]::ParameterValue, 'Show team members and their contributions')
            [CompletionResult]::new('queue', 'queue', [CompletionResultType]::ParameterValue, 'Manage offline operation queue')
            [CompletionResult]::new('comment', 'comment', [CompletionResultType]::ParameterValue, 'Manage comments on commits')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completion scripts')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'auxin;help;init' {
            break
        }
        'auxin;help;add' {
            break
        }
        'auxin;help;commit' {
            break
        }
        'auxin;help;log' {
            break
        }
        'auxin;help;restore' {
            break
        }
        'auxin;help;status' {
            break
        }
        'auxin;help;show' {
            break
        }
        'auxin;help;diff' {
            break
        }
        'auxin;help;compare' {
            break
        }
        'auxin;help;search' {
            break
        }
        'auxin;help;lock' {
            [CompletionResult]::new('acquire', 'acquire', [CompletionResultType]::ParameterValue, 'Acquire exclusive lock for editing')
            [CompletionResult]::new('release', 'release', [CompletionResultType]::ParameterValue, 'Release the lock you currently hold')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current lock status')
            [CompletionResult]::new('break', 'break', [CompletionResultType]::ParameterValue, 'Force break an existing lock (admin only)')
            break
        }
        'auxin;help;lock;acquire' {
            break
        }
        'auxin;help;lock;release' {
            break
        }
        'auxin;help;lock;status' {
            break
        }
        'auxin;help;lock;break' {
            break
        }
        'auxin;help;auth' {
            [CompletionResult]::new('login', 'login', [CompletionResultType]::ParameterValue, 'Login to Oxen Hub with API credentials')
            [CompletionResult]::new('logout', 'logout', [CompletionResultType]::ParameterValue, 'Logout from Oxen Hub')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current authentication status')
            [CompletionResult]::new('test', 'test', [CompletionResultType]::ParameterValue, 'Test authentication with Oxen Hub')
            break
        }
        'auxin;help;auth;login' {
            break
        }
        'auxin;help;auth;logout' {
            break
        }
        'auxin;help;auth;status' {
            break
        }
        'auxin;help;auth;test' {
            break
        }
        'auxin;help;metadata-diff' {
            break
        }
        'auxin;help;daemon' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Check daemon status')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start the daemon service')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop the daemon service')
            [CompletionResult]::new('restart', 'restart', [CompletionResultType]::ParameterValue, 'Restart the daemon service')
            [CompletionResult]::new('logs', 'logs', [CompletionResultType]::ParameterValue, 'Show daemon logs')
            break
        }
        'auxin;help;daemon;status' {
            break
        }
        'auxin;help;daemon;start' {
            break
        }
        'auxin;help;daemon;stop' {
            break
        }
        'auxin;help;daemon;restart' {
            break
        }
        'auxin;help;daemon;logs' {
            break
        }
        'auxin;help;hooks' {
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize hooks directory')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all installed hooks')
            [CompletionResult]::new('builtins', 'builtins', [CompletionResultType]::ParameterValue, 'List available built-in hooks')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Install a built-in hook')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove an installed hook')
            break
        }
        'auxin;help;hooks;init' {
            break
        }
        'auxin;help;hooks;list' {
            break
        }
        'auxin;help;hooks;builtins' {
            break
        }
        'auxin;help;hooks;install' {
            break
        }
        'auxin;help;hooks;remove' {
            break
        }
        'auxin;help;console' {
            break
        }
        'auxin;help;activity' {
            break
        }
        'auxin;help;team' {
            break
        }
        'auxin;help;queue' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show pending operations in the queue')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Manually sync all pending operations')
            [CompletionResult]::new('clear', 'clear', [CompletionResultType]::ParameterValue, 'Clear completed operations from the queue')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a specific operation from the queue')
            break
        }
        'auxin;help;queue;status' {
            break
        }
        'auxin;help;queue;sync' {
            break
        }
        'auxin;help;queue;clear' {
            break
        }
        'auxin;help;queue;remove' {
            break
        }
        'auxin;help;comment' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a comment to a commit')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List comments on a commit')
            break
        }
        'auxin;help;comment;add' {
            break
        }
        'auxin;help;comment;list' {
            break
        }
        'auxin;help;completions' {
            break
        }
        'auxin;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
