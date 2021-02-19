local rofl = require('rofl')

local eq = assert.are.same


local get_file_completions = function(word, cwd)
  local res = rofl._get_completions {
    context = {
      word = word,
      cwd = cwd or vim.loop.cwd(),
    },
    sources = {
      files = true
    }
  }

  table.sort(res)

  return res
end

-- TODO: Should paths show the leading "./" or not?

describe('rofl.nvim files', function()
  it('returns empty list for bad files', function()
    eq({}, get_file_completions('/hello/wor'))
  end)

  it('returns one file when it matches', function()
    -- eq({'./README.md'}, get_file_completions('./README.m'))
    eq({'README.md'}, get_file_completions('./README.m'))
  end)

  it('returns one file when it matches', function()
    eq({'Cargo.lock', 'Cargo.toml'}, get_file_completions('./Car'))
    eq({'Cargo.lock', 'Cargo.toml'}, get_file_completions('./Car'))
  end)

  it('returns files from different cwds', function()
    eq({'file_1.txt', 'file_2.txt', }, get_file_completions('file', './lua/tests/fixtures/cwd_test/'))
  end)
end)
