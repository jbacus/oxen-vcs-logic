#!/usr/bin/osascript
-- Set tempo in an already-opened Logic Pro project
-- Usage: osascript set_tempo_in_project.applescript <tempo>
-- Make sure the project is already open in Logic Pro

on run argv
    if (count of argv) < 1 then
        return "Usage: set_tempo_in_project.applescript <tempo>"
    end if

    set targetTempo to item 1 of argv as number

    tell application "Logic Pro"
        if (count of documents) = 0 then
            return "Error: No project is open in Logic Pro"
        end if

        activate
        delay 1

        -- Try to set tempo directly (may work in some Logic Pro versions)
        try
            tell front document
                set tempo to targetTempo
            end tell
            return "Set tempo to " & targetTempo
        on error
            -- If direct setting doesn't work, use UI automation
            log "Direct tempo setting failed, trying UI automation..."
        end try

        -- UI automation approach
        tell application "System Events"
            tell process "Logic Pro"
                -- Double-click the tempo display
                -- The tempo display is usually in the LCD area at the top

                -- Try keyboard shortcut (Option+T)
                try
                    keystroke "t" using {option down}
                    delay 0.5

                    -- Clear and enter new tempo
                    keystroke "a" using command down
                    keystroke (targetTempo as string)
                    keystroke return

                    return "Set tempo to " & targetTempo & " (via UI)"
                on error errMsg
                    return "Error setting tempo: " & errMsg
                end try
            end tell
        end tell
    end tell
end run
