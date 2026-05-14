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

function _memy_select
    set selected (memy --color always list $argv -s)
    if test -z "$selected"
        return 1
    end
    echo (string replace -r '^~' $HOME -- $selected)
end

function memy-cd
    set selected (_memy_select -d); or return
    cd $selected
end

function memy-open
    set selected (_memy_select -f); or return
    memy open $selected
end

function memy-go
    set selected (_memy_select); or return
    if test -d "$selected"
        cd $selected
    else
        memy open $selected
    end
end

if not functions -q z; and not command -q z
    function z
        if test (count $argv) -eq 1 -a "$argv[1]" = "-"
            cd $OLDPWD
            return
        end
        set result (memy z -- $argv)
        and cd $result
    end
end

if not functions -q zi; and not command -q zi
    function zi
        set result (memy z -i -- $argv)
        and cd $result
    end
end
