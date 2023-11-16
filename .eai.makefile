docs: ## Build documentation
	@if [[ ! -d docs ]]; then \
		git worktree add docs gh-pages; \
	fi
	@cargo doc --no-deps && cp -r target/doc/*

docs-repair:
	mkdir -p docs/build/html || true; \
	git worktree repair docs/build/html; \
	(cd docs/build/html && git pull)

docs-publish: ## Push built documentation to gh-pages branch
	@cd docs/build/html && \
	git add --all && \
	git commit -m'Documentation update $(shell date)' && \
	git push origin gh-pages

docs-clean: ## Clear documentation build artifacts
	@make -C docs clean

