local rofl = require('rofl')

local eq = assert.are.same

local get_context = function(word)
  return {
    word = word,
    cwd = vim.loop.cwd(),
    bufnr = vim.api.nvim_get_current_buf(),
  }
end

local get_buffer_completions = function(word, disable_buffer)
  local res = rofl._get_completions {
    context = get_context(word),
    sources = {
      buffer = not disable_buffer,
      file = true,
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

    -- local bufnr = vim.api.nvim_get_current_buf()
    rofl.attach(bufnr)
  end)

  it('returns nothing in an empty buffer', function()
    eq({}, get_buffer_completions("not_a_prefix"))
    eq({}, get_buffer_completions("not_a_prefix"))
    eq({}, get_buffer_completions("not_a_prefix"))
    eq({}, get_buffer_completions("not_a_prefix"))
    eq({}, get_buffer_completions("not_a_prefix"))
  end)

  it('returns things only from the current buffer', function()
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

  it('returns valid candidates in a file', function()
    vim.api.nvim_buf_set_lines(0, 0, -1, false, {"hello", "world"})
    eq({"hello", "world"}, vim.api.nvim_buf_get_lines(0, 0, -1, false))

    eq({}, get_buffer_completions("not_a_thing"))
    eq({"hello"}, get_buffer_completions("hel"))
    eq({"world"}, get_buffer_completions("w"))

    vim.api.nvim_buf_set_lines(0, 0, -1, false, {"", "world"})

    eq({}, get_buffer_completions("not_a_thing"))
    eq({}, get_buffer_completions("hel"))
    eq({"world"}, get_buffer_completions("w"))
    eq({}, get_buffer_completions("w", true))
    eq({"world"}, get_buffer_completions("w", false))
  end)

end)
