
function! MyTestFunction(findstart, base)
  if a:findstart
    return col('.')
  else
    call timer_start(500, { -> complete_add('another') })

    return ['hello', 'world']
  end
endfunction

set completefunc=MyTestFunction
inoremap <c-t> <c-r>=MyTestFunction()<CR>


fun! CompleteMonths(findstart, base)
  if a:findstart
    " locate the start of the word
    let line = getline('.')
    let start = col('.') - 1
    while start > 0 && line[start - 1] =~ '\a'
      let start -= 1
    endwhile
    return start
  else
    " find months matching with "a:base"
    for m in split("Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec")
      if m =~ '^' . a:base
    call complete_add(m)
      endif
      sleep 300m	" simulate searching for next match
      if complete_check()
    break
      endif
    endfor
    return []
  endif
endfun
set completefunc=CompleteMonths



