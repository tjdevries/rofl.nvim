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
  before_each(function()
    vim.api.nvim_buf_delete(0, { force = true })
    local bufnr = vim.api.nvim_create_buf(true, false)
    vim.api.nvim_set_current_buf(bufnr)

    rofl.attach(bufnr)
  end)

  it('returns empty list for bad files', function()
    vim.api.nvim_buf_set_lines(0, 0, -1, false, {"hello", "world"})
    eq({"hello", "world"}, vim.api.nvim_buf_get_lines(0, 0, -1, false))
  end)

  it('can return only from current buffer', function()
    --[[
      hello world

      ----------------

      goodnight moon

    --]]

    --[[    ^n -> rofl.compete { buffers = { current_buf } },
    --      ^x^n -> rofl.complete { buffers = { all_visible } }
    --      <M-n> -> rofl.complete { buffers = { all } }
    --]]
  end)
end)
