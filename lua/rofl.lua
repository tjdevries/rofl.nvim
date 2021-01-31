local api = vim.api
local rofl = {}

-- local binary_path = vim.fn.fnamemodify(api.nvim_get_runtime_file("lua/rofl.lua", false)[1], ":h:h") .. "/target/debug/rofl_nvim"
local binary_path = vim.fn.fnamemodify(api.nvim_get_runtime_file("lua/rofl.lua", false)[1], ":h:h") .. "/target/release/rofl_nvim"

rofl.start = function(bufnr)
  bufnr = bufnr or 0

  if rofl.job_id then
    return
  end

  rofl.job_id = vim.fn.jobstart(
    {binary_path},
    {
      rpc = true
    }
  )
end

rofl.attach = function(bufnr)
  bufnr = bufnr or 0

  vim.cmd [[autocmd! InsertCharPre <buffer> lua require'rofl'.notify("v_char", vim.api.nvim_get_vvar("char"))]]

  vim.cmd [[autocmd! InsertLeave <buffer> lua require'rofl'.notify("insert_leave")]]

  api.nvim_buf_attach(bufnr, true, {
    on_lines = function()
      -- local mode =  api.nvim_get_mode()["mode"]
      rofl.notify("complete")
    end,
  })
end

rofl.request = function(method, ...)
  rofl.start()
  return vim.rpcrequest(rofl.job_id, method, ...)
end

rofl.notify = function(method, ...)
  rofl.start()
  vim.rpcnotify(rofl.job_id, method, ...)
end

local sources = {
  current = 1,
  fns = {},
}

rofl.add_source = function(fn)
  rofl.start()
  table.insert(sources.fns, fn)
end

-- use this to be able to run sources in tokio tasks
rofl.step_source = function()
  rofl.start()
  local res = sources.fns[sources.current]()
  sources.current = sources.current + 1
  if sources.current > #sources.fns then
    sources.current = 1
  end
  return res
end

rofl.step_amount = function()
  rofl.start()
  return #sources.fns
end

return rofl
