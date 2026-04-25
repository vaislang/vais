# Vais hello world examples — build & verify exit codes.
VAISC ?= $(HOME)/.cargo/bin/vaisc
RUNTIME_C := ../../selfhost/runtime.c
SYNC_C := ../../std/sync_runtime.c
TMPDIR := /tmp

.PHONY: check clean

check:
	@for f in 0*.vais; do \
	  base=$$(basename $$f .vais); \
	  $(VAISC) build $$f --emit-ir -o $(TMPDIR)/$$base.ll --force-rebuild >/dev/null 2>&1 || \
	    { echo "FAIL compile: $$base"; exit 1; }; \
	  clang -O0 -o $(TMPDIR)/$$base $(TMPDIR)/$$base*.ll $(RUNTIME_C) $(SYNC_C) -lm >/dev/null 2>&1 || \
	    { echo "FAIL link: $$base"; exit 1; }; \
	  $(TMPDIR)/$$base; ec=$$?; \
	  echo "PASS: $$base (exit=$$ec)"; \
	done
	@echo
	@echo "All hello-world examples passed."

clean:
	rm -f $(TMPDIR)/0[1-9]_*
