#!/usr/bin/env zsh
# Leuwi Panjang Terminal - ZSH Prompt
# Simple, informative: user@host dir (git:branch ±N) $/#

setopt PROMPT_SUBST

_lp_git_info() {
    local branch
    branch=$(git symbolic-ref --short HEAD 2>/dev/null) || return

    local changes=0
    local staged=$(git diff --cached --numstat 2>/dev/null | wc -l)
    local unstaged=$(git diff --numstat 2>/dev/null | wc -l)
    local untracked=$(git ls-files --others --exclude-standard 2>/dev/null | wc -l)
    changes=$((staged + unstaged + untracked))

    local icon=""  # Nerd Font git branch icon
    if [[ $changes -eq 0 ]]; then
        echo " %F{green}${icon} ${branch} ✓%f"
    else
        echo " %F{yellow}${icon} ${branch} ±${changes}%f"
    fi
}

_lp_prompt_char() {
    if [[ $EUID -eq 0 ]]; then
        echo "#"
    else
        echo "$"
    fi
}

precmd() {
    local exit_code=$?
    local git_info=$(_lp_git_info)
    local pchar=$(_lp_prompt_char)

    # Color prompt char based on last exit code
    local pcolor
    if [[ $exit_code -eq 0 ]]; then
        pcolor="%F{green}"
    else
        pcolor="%F{red}"
    fi

    PROMPT="%F{green}%n%f%F{white}@%f%F{cyan}%m%f %F{yellow}%~%f${git_info} ${pcolor}${pchar}%f "
    RPROMPT="%F{8}%T%f"
}
