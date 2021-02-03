
function! RoflComplete(findstart, base)
  call luaeval("RELOAD('rofl')")

  return luaeval('require("rofl").complete_func(_A.findstart, _A.base)', {
        \ "findstart": a:findstart,
        \ "base": a:base,
        \ })
endfunction


