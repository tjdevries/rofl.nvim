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

  -- local function on_lines(bufnr, firstline, new_lastline)
  --   if api.nvim_get_mode()["mode"]:find("i") == nil then
  --     return
  --   end

  --   firstline = api.nvim_buf_get_lines(bufnr, firstline, new_lastline, false)

  --   rofl.notify("on_lines", firstline)
  -- end

  -- api.nvim_buf_attach(bufnr, false, {
  --   on_lines = function(_, bufnr, firstline, lastline, new_lastline)
  --     local status, err = pcall(on_lines, bufnr, firstline, new_lastline)
  --     if err then
  --       error(err)
  --       return true
  --     end
  --   end
  -- })
end

rofl.attach = function(bufnr)
  bufnr = bufnr or 0
  api.nvim_buf_attach(bufnr, true, {
    on_lines = function()
      print("notifying")
      rofl.notify("complete")
    end
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

rofl.test = function()
  rofl.start()

  print("Sending a request...")
  print("Result:", rofl.request("first", 1))
  print("Done!")

  print("NOTIFY")
  rofl.notify("PogChamp")
end

return rofl
