#!/usr/bin/env zsh
# Leuwi Panjang Terminal - ZSH Prompt Integration
# Lightweight, colorful, informative prompt
#
# Install: source this file in ~/.zshrc
#   source /path/to/leuwi-prompt.zsh

# Colors (256-color mode)
_LP_RST="%f%k%b"      # reset
_LP_GREEN="%F{#00FF88}"
_LP_DIM_GREEN="%F{#5C8A72}"
_LP_CYAN="%F{#00B4D8}"
_LP_YELLOW="%F{#FFD166}"
_LP_RED="%F{#EF476F}"
_LP_MAGENTA="%F{#C77DFF}"
_LP_WHITE="%F{#B8D4CC}"
_LP_DIM="%F{#3A5A48}"
_LP_BOLD="%B"

# Git info function
_lp_git_info() {
    local branch
    branch=$(git symbolic-ref --short HEAD 2>/dev/null || git rev-parse --short HEAD 2>/dev/null)
    [[ -z "$branch" ]] && return

    local status_icon=""
    local git_status
    git_status=$(git status --porcelain 2>/dev/null)

    if [[ -z "$git_status" ]]; then
        status_icon="${_LP_GREEN}✓${_LP_RST}"
    else
        local staged=$(echo "$git_status" | grep -c "^[MADRC]")
        local unstaged=$(echo "$git_status" | grep -c "^.[MADRC]")
        local untracked=$(echo "$git_status" | grep -c "^??")

        [[ $staged -gt 0 ]] && status_icon+="${_LP_GREEN}+${staged}${_LP_RST}"
        [[ $unstaged -gt 0 ]] && status_icon+="${_LP_YELLOW}~${unstaged}${_LP_RST}"
        [[ $untracked -gt 0 ]] && status_icon+="${_LP_DIM}?${untracked}${_LP_RST}"
    fi

    # Ahead/behind
    local ahead behind
    ahead=$(git rev-list --count @{upstream}..HEAD 2>/dev/null)
    behind=$(git rev-list --count HEAD..@{upstream} 2>/dev/null)
    local sync=""
    [[ "$ahead" -gt 0 ]] && sync+="${_LP_CYAN}↑${ahead}${_LP_RST}"
    [[ "$behind" -gt 0 ]] && sync+="${_LP_RED}↓${behind}${_LP_RST}"

    echo " ${_LP_DIM}on${_LP_RST} ${_LP_MAGENTA} ${branch}${_LP_RST} ${status_icon}${sync}"
}

# OS icon
_lp_os_icon() {
    if [[ -f /etc/os-release ]]; then
        local id=$(. /etc/os-release && echo "$ID")
        case "$id" in
            ubuntu)     echo "" ;;
            debian)     echo "" ;;
            arch)       echo "" ;;
            fedora)     echo "" ;;
            centos)     echo "" ;;
            alpine)     echo "" ;;
            opensuse*)  echo "" ;;
            manjaro)    echo "" ;;
            *)          echo "" ;;
        esac
    elif [[ "$(uname)" == "Darwin" ]]; then
        echo ""
    else
        echo ""
    fi
}

# Shorten directory path
_lp_short_dir() {
    local dir="${PWD/#$HOME/~}"
    # If path is long, abbreviate middle parts
    if [[ ${#dir} -gt 40 ]]; then
        echo "${dir:0:15}…${dir: -22}"
    else
        echo "$dir"
    fi
}

# Build prompt
_lp_build_prompt() {
    local exit_code=$?
    local os_icon=$(_lp_os_icon)
    local short_dir=$(_lp_short_dir)
    local git_info=$(_lp_git_info)

    # Prompt indicator color based on last exit code
    local indicator
    if [[ $exit_code -eq 0 ]]; then
        indicator="${_LP_GREEN}❯${_LP_RST}"
    else
        indicator="${_LP_RED}❯${_LP_RST}"
    fi

    # Line 1: [padding] os_icon user@host  directory  git_info
    # Line 2: [padding] ❯ (prompt)
    echo ""  # Empty line for padding before prompt
    PROMPT="${_LP_DIM}${os_icon}${_LP_RST} ${_LP_GREEN}${_LP_BOLD}%n${_LP_RST}${_LP_DIM}@${_LP_RST}${_LP_CYAN}%m${_LP_RST}  ${_LP_YELLOW}${short_dir}${_LP_RST}${git_info}
${indicator} "
}

# Right prompt: timestamp
RPROMPT="${_LP_DIM}%T${_LP_RST}"

# Hook into precmd to rebuild prompt each time
precmd_functions+=(_lp_build_prompt)

# Enable prompt substitution
setopt PROMPT_SUBST

# Initial build
_lp_build_prompt
