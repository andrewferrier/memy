function fish_preexec --on-event fish_preexec
    set cmd $argv[1]

    for word in (eval printf '%s\n' $cmd)
        set expanded (eval echo $word)

        if test -e "$expanded"
            memy  \
                --config denied_files_warn_on_note=false \
                --config missing_files_warn_on_note=false \
                note "$expanded" &
        end
    end
end

function memy-cd
    set selected (memy list -d | fzf)
    if test -n "$selected"
        set selected (string replace -r '^~' $HOME $selected)
        cd "$selected"
    end
end
