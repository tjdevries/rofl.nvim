local rofl = require('rofl')

local eq = assert.are.same

describe('rofl.nvim connection', function()
  it('can connect and send a test request', function()
    eq(true, rofl.request("_test"))
  end)
end)
