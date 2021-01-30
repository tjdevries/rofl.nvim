local api = vim.api
local rofl = {}

local binary_path = vim.fn.fnamemodify(api.nvim_get_runtime_file("lua/rofl.lua", false)[1], ":h:h") .. "/target/debug/rofl_nvim"

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

  vim.cmd [[autocmd! InsertCharPre <buffer> lua require'rofl'.send_char()]]

  api.nvim_buf_attach(bufnr, true, {
    on_lines = function()
      -- local mode =  api.nvim_get_mode()["mode"]
      rofl.notify("complete")
    end,
  })
end

rofl.send_char = function()
  rofl.notify("v_char", api.nvim_get_vvar("char"))
end

rofl.request = function(method, ...)
  rofl.start()
  return vim.rpcrequest(rofl.job_id, method, ...)
end

rofl.notify = function(method, ...)
  rofl.start()
  vim.rpcnotify(rofl.job_id, method, ...)
end

rofl.test = function()
  rofl.start()

  print("Sending a request...")
  print("Result:", rofl.request("first", 1))
  print("Done!")

  print("NOTIFY")
  rofl.notify("PogChamp")
end

return rofl
