if (($+commands[memy])); then
    eval "$(memy hook zsh)"
else
    echo 'memy: command not found, please install it from https://github.com/andrewferrier/memy'
fi
