local api = vim.api
local rofl = {}

local binary_path = vim.fn.fnamemodify(api.nvim_get_runtime_file("lua/rofl.lua", false)[1], ":h:h") .. "/target/debug/rofl_nvim"

rofl.start = function()
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
