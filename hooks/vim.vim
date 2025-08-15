augroup MyMemyNote
  autocmd!
  autocmd BufReadPre,BufWritePost * call s:RunMemyNote(expand('<amatch>'))
augroup END

function! s:RunMemyNote(file) abort
  let l:file = a:file

  call job_start(['memy', 'note', l:file], {
        \ 'exit_cb': function('s:OnMemyExit')
        \ })
endfunction

function! s:OnMemyExit(job_id, exit_info) abort
  if type(a:exit_info) == type({}) && has_key(a:exit_info, 'exit_status')
    let l:code = a:exit_info.exit_status
  else
    let l:code = a:exit_info
  endif

  if l:code != 0
    echohl ErrorMsg
    echom 'From external command: exited with code ' . l:code
    echohl None
  endif
endfunction
