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

local function throttle_leading(fn, ms)
  local timer = vim.loop.new_timer()
  local running = false

  local function wrapped_fn()
    if not running then
      timer:start(ms, 0, function()
        running = false
      end)
      running = true
      pcall(vim.schedule_wrap(fn))
    end
  end
  return wrapped_fn, timer
end

do
  local throttled, timer = throttle_leading(function() rofl.notify("compelte") end, 10)
  rofl.attach = function(bufnr)
    bufnr = bufnr or 0
    api.nvim_buf_attach(bufnr, true, {
      on_lines = function()
        rofl.notify("complete")
      end,
      -- on_lines = throttled,
      on_detach = function()
        timer:close()
      end
    })
  end
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
