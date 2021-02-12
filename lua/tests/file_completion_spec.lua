local rofl = require('rofl')

local eq = assert.are.same


local get_file_completions = function(word)
  return rofl._get_completions {
    context = {
      word = word,
      cwd = vim.loop.cwd(),
    },
    sources = {
      files = true
    }
  }
end

describe('rofl.nvim files', function()
  it('returns empty list for bad files', function()
    eq({}, get_file_completions('/hello/wor'))
  end)

  it('returns one file when it matches', function()
    eq({'./README.md'}, get_file_completions('./README.m'))
  end)

  it('returns one file when it matches', function()
    eq({'./Cargo.toml', './Cargo.lock'}, get_file_completions('./Car'))
  end)
end)
