#!/usr/bin/env fish
# Leuwi Panjang Terminal - Fish Prompt Integration
# Lightweight, colorful, informative prompt
#
# Install: source this file in ~/.config/fish/config.fish
#   source /path/to/leuwi-prompt.fish

function _lp_os_icon
    if test -f /etc/os-release
        set -l id (grep '^ID=' /etc/os-release | cut -d= -f2)
        switch $id
            case ubuntu;  echo ""
            case debian;  echo ""
            case arch;    echo ""
            case fedora;  echo ""
            case centos;  echo ""
            case alpine;  echo ""
            case manjaro; echo ""
            case '*';     echo ""
        end
    else if test (uname) = "Darwin"
        echo ""
    else
        echo ""
    end
end

function _lp_git_info
    set -l branch (git symbolic-ref --short HEAD 2>/dev/null; or git rev-parse --short HEAD 2>/dev/null)
    test -z "$branch"; and return

    set -l git_status (git status --porcelain 2>/dev/null)
    set -l status_icon ""

    if test -z "$git_status"
        set status_icon (set_color green)"✓"(set_color normal)
    else
        set -l staged (echo "$git_status" | grep -c "^[MADRC]")
        set -l unstaged (echo "$git_status" | grep -c "^.[MADRC]")
        set -l untracked (echo "$git_status" | grep -c "^??")

        test $staged -gt 0; and set status_icon $status_icon(set_color green)"+$staged"(set_color normal)
        test $unstaged -gt 0; and set status_icon $status_icon(set_color yellow)"~$unstaged"(set_color normal)
        test $untracked -gt 0; and set status_icon $status_icon(set_color 5C8A72)"?$untracked"(set_color normal)
    end

    echo " "(set_color 5C8A72)"on"(set_color normal)" "(set_color magenta)" $branch"(set_color normal)" $status_icon"
end

function fish_prompt
    set -l last_status $status
    set -l os_icon (_lp_os_icon)
    set -l git_info (_lp_git_info)

    # Prompt indicator color
    if test $last_status -eq 0
        set -l indicator (set_color 00FF88)"❯"(set_color normal)
    else
        set -l indicator (set_color red)"❯"(set_color normal)
    end

    echo ""
    echo -n (set_color 5C8A72)$os_icon(set_color normal)" "
    echo -n (set_color --bold 00FF88)$USER(set_color normal)
    echo -n (set_color 5C8A72)"@"(set_color normal)
    echo -n (set_color cyan)(hostname -s)(set_color normal)
    echo -n "  "
    echo -n (set_color yellow)(prompt_pwd)(set_color normal)
    echo -n $git_info
    echo ""

    if test $last_status -eq 0
        echo -n (set_color 00FF88)"❯ "(set_color normal)
    else
        echo -n (set_color red)"❯ "(set_color normal)
    end
end

function fish_right_prompt
    echo -n (set_color 5C8A72)(date +%H:%M)(set_color normal)
end
