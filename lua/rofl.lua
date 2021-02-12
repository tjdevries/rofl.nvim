local api = vim.api

local rofl = {}

local binary_path = vim.fn.fnamemodify(
  api.nvim_get_runtime_file("lua/rofl.lua", false)[1], ":h:h")
  .. "/target/debug/rofl_nvim"

if 0 == vim.fn.executable(binary_path) then
  binary_path = vim.fn.fnamemodify(
    api.nvim_get_runtime_file("lua/rofl.lua", false)[1], ":h:h")
    .. "/target/release/rofl_nvim"
end

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
  print("Making new job...", rofl.job_id)
  print("Request Result: ", rofl.request(
    "buf_initialize",
    vim.api.nvim_get_current_buf(),
    vim.bo.iskeyword
  ))
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
  local result = vim.rpcrequest(rofl.job_id, method, ...)
  print("Result:", method, result)
  return result
end



rofl.complete_func = function(find_start, base)
  if find_start == 1 then
    return rofl.request(
      'find_start',

      -- Current bufnr
      vim.api.nvim_get_current_buf(),

      -- Current line
      vim.api.nvim_get_current_line(),

      -- Cursor column
      vim.api.nvim_win_get_cursor(0)[2]
    )
  else
    return rofl.request(
      'complete',

      -- And more...
      base
    )
  end
end

vim.bo.completefunc = 'RoflComplete'

rofl.notify = function(method, ...)
  rofl.start()
  vim.rpcnotify(rofl.job_id, method, ...)
end

rofl._get_completions = function(req)
  return rofl.request(
    'complete_sync',

    req.context or rofl._get_context(),
    req.sources or rofl._get_sources()
  )
end

return rofl
