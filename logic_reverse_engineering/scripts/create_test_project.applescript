#!/usr/bin/osascript
-- Create a Logic Pro test project with specific settings
-- Usage: osascript create_test_project.applescript <project_name> <tempo> <sample_rate> <key> <time_sig>

on run argv
    if (count of argv) < 5 then
        display dialog "Usage: create_test_project.applescript <name> <tempo> <sample_rate> <key> <time_sig>" buttons {"OK"} default button 1
        return
    end if

    set projectName to item 1 of argv
    set targetTempo to item 2 of argv as number
    set targetSampleRate to item 3 of argv as number
    set targetKey to item 4 of argv
    set targetTimeSig to item 5 of argv

    -- Parse time signature (e.g., "4/4" -> numerator=4, denominator=4)
    set AppleScript's text item delimiters to "/"
    set timeSigParts to text items of targetTimeSig
    set timeSigNum to item 1 of timeSigParts as number
    set timeSigDenom to item 2 of timeSigParts as number
    set AppleScript's text item delimiters to ""

    -- Get projects directory path
    set projectsDir to (do shell script "cd " & quoted form of (POSIX path of (path to me)) & "/../projects && pwd")
    set savePath to projectsDir & "/" & projectName & ".logicx"

    tell application "Logic Pro"
        activate

        -- Create new empty project
        make new document
        delay 2

        -- Get the front document
        tell front document
            -- Set tempo
            try
                set tempo to targetTempo
                log "Set tempo to " & targetTempo
            on error errMsg
                log "Warning: Could not set tempo - " & errMsg
            end try

            -- Set time signature
            try
                -- Logic Pro's time signature might need different approach
                -- This may vary by version
                log "Time signature: " & timeSigNum & "/" & timeSigDenom
            on error errMsg
                log "Warning: Could not set time signature - " & errMsg
            end try

            -- Add one software instrument track
            try
                make new software instrument track
                log "Added software instrument track"
            on error errMsg
                log "Warning: Could not add track - " & errMsg
            end try

            -- Save the project
            try
                save in POSIX file savePath
                log "Saved project to: " & savePath
            on error errMsg
                display dialog "Failed to save: " & errMsg buttons {"OK"} default button 1
            end try
        end tell

        -- Close the project
        delay 1
        close front document saving no

    end tell

    return "Created project: " & projectName
end run
