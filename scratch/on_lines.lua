
vim.api.nvim_buf_attach(0, false, {
  --[[
      args.items[2] = INTEGER_OBJ(start_row);
      args.items[3] = INTEGER_OBJ(start_col);
      args.items[4] = INTEGER_OBJ(start_byte);
      args.items[5] = INTEGER_OBJ(old_row);
      args.items[6] = INTEGER_OBJ(old_col);
      args.items[7] = INTEGER_OBJ(old_byte);
      args.items[8] = INTEGER_OBJ(new_row);
      args.items[9] = INTEGER_OBJ(new_col);
      args.items[10] = INTEGER_OBJ(new_byte);
  --]]
  -- on_bytes = function(_, bufnr, start_row, start_col, start_byte, old_row, old_col, old_byte, new_row, new_col, new_byte)
  --   P({...})
  -- end
  on_lines = function(...)
    P({...})
  end,
})


