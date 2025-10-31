#!/usr/bin/osascript
-- Create Logic Pro test project using UI automation
-- This uses System Events to control Logic Pro's UI
-- More reliable than direct AppleScript commands

-- IMPORTANT: Enable "System Preferences > Security & Privacy > Accessibility"
-- and allow Terminal or Script Editor to control your computer

on run argv
    if (count of argv) < 2 then
        return "Usage: osascript create_test_project_ui.applescript <project_name> <tempo>"
    end if

    set projectName to item 1 of argv
    set targetTempo to item 2 of argv

    -- Get projects directory
    set projectsDir to (do shell script "cd " & quoted form of (POSIX path of (path to me)) & "/../projects && pwd")

    tell application "Logic Pro"
        activate
        delay 2

        -- Create new project
        tell application "System Events"
            tell process "Logic Pro"
                -- File > New
                keystroke "n" using command down
                delay 3

                -- In the new project dialog, choose "Empty Project"
                -- This part may need adjustment based on Logic Pro's dialog
                try
                    click button "Empty Project" of window 1
                    delay 1
                    click button "Choose" of window 1
                    delay 2
                on error
                    log "Could not click Empty Project button"
                end try

                -- Create dialog for track - press Cancel to skip
                try
                    keystroke return -- or press Cancel
                    delay 1
                on error
                    log "No track dialog"
                end try
            end tell
        end tell

        -- Now set the tempo using UI automation
        tell application "System Events"
            tell process "Logic Pro"
                -- Click on tempo display (usually top center)
                -- This is tricky - may need coordinates

                -- Alternative: Use keyboard shortcut
                -- Option+T opens tempo/time signature
                keystroke "t" using {option down}
                delay 1

                -- Clear current value and type new tempo
                keystroke "a" using command down
                keystroke targetTempo
                keystroke return
                delay 1
            end tell
        end tell

        -- Save the project
        tell application "System Events"
            tell process "Logic Pro"
                -- Cmd+S to save
                keystroke "s" using command down
                delay 2

                -- In save dialog, set filename
                try
                    set value of text field 1 of sheet 1 of window 1 to projectName
                    delay 1

                    -- Navigate to projects directory
                    keystroke "g" using {command down, shift down}
                    delay 1
                    keystroke projectsDir
                    keystroke return
                    delay 1

                    -- Click Save
                    click button "Save" of sheet 1 of window 1
                    delay 2
                on error errMsg
                    log "Save error: " & errMsg
                end try
            end tell
        end tell

        -- Close the project
        tell application "System Events"
            tell process "Logic Pro"
                keystroke "w" using command down
                delay 1
            end tell
        end tell
    end tell

    return "Created project: " & projectName & " with tempo " & targetTempo
end run
