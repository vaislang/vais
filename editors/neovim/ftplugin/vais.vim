" Vais filetype plugin
" Language: Vais
" Maintainer: Vais Language Team

if exists("b:did_ftplugin")
  finish
endif
let b:did_ftplugin = 1

" Save the compatible options
let s:save_cpo = &cpo
set cpo&vim

" Comment string
setlocal commentstring=#\ %s

" Indentation settings
setlocal expandtab        " Use spaces instead of tabs
setlocal shiftwidth=2     " Number of spaces for each indentation level
setlocal softtabstop=2    " Number of spaces for a tab
setlocal tabstop=2        " Number of spaces for a tab character
setlocal autoindent       " Copy indent from current line when starting a new line
setlocal smartindent      " Smart indenting for C-like languages

" Format options
setlocal formatoptions-=t " Don't auto-wrap text
setlocal formatoptions+=c " Auto-wrap comments
setlocal formatoptions+=r " Insert comment leader after <Enter> in insert mode
setlocal formatoptions+=o " Insert comment leader after 'o' or 'O' in normal mode
setlocal formatoptions+=q " Allow formatting comments with 'gq'
setlocal formatoptions+=n " Recognize numbered lists
setlocal formatoptions+=j " Remove comment leader when joining lines

" Matching pairs
setlocal matchpairs+=<:>

" Folding (optional - based on indentation)
setlocal foldmethod=indent
setlocal foldnestmax=10
setlocal nofoldenable      " Don't fold by default
setlocal foldlevel=2

" Undo ftplugin settings
let b:undo_ftplugin = "setlocal commentstring< expandtab< shiftwidth< softtabstop< tabstop<"
      \ . " | setlocal autoindent< smartindent< formatoptions< matchpairs<"
      \ . " | setlocal foldmethod< foldnestmax< foldenable< foldlevel<"

" Restore compatible options
let &cpo = s:save_cpo
unlet s:save_cpo
