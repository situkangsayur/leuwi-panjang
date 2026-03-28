#!/usr/bin/env zsh
# Leuwi Panjang Terminal - ZSH Prompt
# Simple, informative: user@host dir (git:branch ±N) $/#

setopt PROMPT_SUBST

# ── Color ls output ──
alias ls='ls --color=auto'
alias ll='ls -lah --color=auto'
alias la='ls -A --color=auto'

# Rich LS_COLORS for colored file listings
export LS_COLORS='di=1;34:ln=36:so=35:pi=33:ex=1;32:bd=1;33;40:cd=1;33;40:su=37;41:sg=30;43:tw=30;42:ow=34;42:*.tar=31:*.gz=31:*.zip=31:*.7z=31:*.deb=31:*.rpm=31:*.jpg=35:*.jpeg=35:*.png=35:*.gif=35:*.svg=35:*.mp4=35:*.mkv=35:*.mp3=35:*.flac=35:*.pdf=33:*.doc=33:*.rs=38;5;208:*.go=38;5;81:*.py=38;5;226:*.js=38;5;220:*.ts=38;5;45:*.java=38;5;166:*.kt=38;5;99:*.sh=38;5;113:*.md=38;5;248:*.toml=38;5;173:*.yaml=38;5;173:*.json=38;5;178:*.html=38;5;166:*.css=38;5;75:*.php=38;5;99'

# If eza is available, use it with icons
if command -v eza &>/dev/null; then
    alias ls='eza --icons --color=auto'
    alias ll='eza -lah --icons --color=auto --git'
    alias la='eza -A --icons --color=auto'
    alias lt='eza --tree --icons --color=auto --level=2'
fi

# Grep colors
alias grep='grep --color=auto'
alias fgrep='fgrep --color=auto'
alias egrep='egrep --color=auto'

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
