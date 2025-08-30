function fish_preexec --on-event fish_preexec
    set cmd $argv[1]

    for word in (eval printf '%s\n' $cmd)
        set expanded (eval echo $word)

        if test -e "$expanded"
            memy --config denied_files_warn_on_note=false note "$expanded" &
        end
    end
end
