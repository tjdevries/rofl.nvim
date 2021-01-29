local rofl = require('rofl')

local eq = assert.are.same

describe('rofl.nvim connection', function()
  it('can connect and send a test request', function()
    eq(true, rofl.request("_test"))
  end)

  it('can get the first line with complete', function()
    vim.api.nvim_buf_set_lines(0, 0, -1, false, {"hello"})
    eq("hello", rofl.request("complete"))
  end)
end)
