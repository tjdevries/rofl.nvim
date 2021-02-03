
function! RoflComplete(findstart, base)
  return luaeval('require("rofl").complete_func(_A.findstart, _A.base)', {
        \ "findstart": a:findstart,
        \ "base": a:base,
        \ })
endfunction


