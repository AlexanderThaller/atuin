set -gx ATUIN_SESSION (atuin uuid)

function _atuin_preexec --on-event fish_preexec
    if not test -n "$fish_private_mode"
        set -gx ATUIN_HISTORY_ID (atuin history start -- "$argv[1]")
    end
end

function _atuin_postexec --on-event fish_postexec
    set s $status

    if test -n "$ATUIN_HISTORY_ID"
        ATUIN_LOG=error atuin history end --exit $s -- $ATUIN_HISTORY_ID &>/dev/null &
        disown
    end

    set --erase ATUIN_HISTORY_ID
end

function _atuin_search
    set h (ATUIN_SHELL_FISH=t ATUIN_LOG=error atuin search $argv -i -- (commandline -b) 3>&1 1>&2 2>&3)

    if test -n "$h"
        if string match -qg '__atuin_accept__:*' "$h"
          set hist (string match -r '__atuin_accept__:(.*|\s*)' "$h"  | awk 'NR == 2')
          echo $hist
          # Need to run the pre/post exec functions manually
          _atuin_preexec $hist
          eval $hist
          _atuin_postexec
          # Allow space for repainting the prompt, this will work for prompts up to 2 lines
          echo
          echo
        else
          commandline -r "$h"
        end
    end

    commandline -f repaint
end

function _atuin_bind_up
    # Fallback to fish's builtin up-or-search if we're in search or paging mode
    if commandline --search-mode; or commandline --paging-mode
        up-or-search
        return
    end

    # Only invoke atuin if we're on the top line of the command
    set -l lineno (commandline --line)

    switch $lineno
        case 1
            _atuin_search --shell-up-key-binding
        case '*'
            up-or-search
    end
end
