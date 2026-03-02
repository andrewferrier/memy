function fish_preexec --on-event fish_preexec
    set cmd $argv[1]

    for word in (string match -ra '\S+' -- $cmd)
        set expanded $word

        if string match -q '~*' -- $word
            set expanded (string replace -r '^~' $HOME $word)
        end

        if test -e "$expanded"
            memy  \
                --config denied_files_warn_on_note=false \
                --config missing_files_warn_on_note=false \
                note "$expanded" &
        end
    end
end

function memy-cd
    set selected (memy list -d -s)
    if test -n "$selected"
        cd "$selected"
    end
end
