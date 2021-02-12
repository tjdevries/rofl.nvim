
local rofl = require('rofl')

function DoTest()
  return rofl._get_completions {
    context = {
      word = vim.fn.expand("<cWORD>"),
      cwd = vim.loop.cwd(),
    },
    sources = {
      files = true
    }
  }
end

function CompleteFile()
  local line = vim.api.nvim_get_current_line()
  local col = vim.fn.col('.')

  local word = line:sub(1, col)

  local completions = rofl._get_completions {
    context = {
      word = word,
      cwd = vim.loop.cwd(),
    },
    sources = {
      files = true
    }
  }

  vim.fn.complete(1, completions)

  return ''
end

vim.cmd [[inoremap <c-x><c-f> <c-r>=luaeval('CompleteFile()')<CR>]]
