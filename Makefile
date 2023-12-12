.PHONY: all help docs docs-repair docs-publish docs-clean

all: help

docs: ## Build documentation
	@if [[ ! -d docs ]]; then \
		git worktree add docs gh-pages; \
	fi
	@cargo doc --no-deps --all-features && cp -r target/doc/* docs/
	@echo "<meta http-equiv=\"refresh\" content=\"0; url=$(subst -,_,$(NAME))\">" > docs/index.html
	@touch docs/.nojekyll

docs-repair:
	mkdir -p docs || true; \
	git worktree repair docs; \
	(cd docs && git pull)

docs-publish: ## Push built documentation to gh-pages branch
	@cd docs && \
	git add --all && \
	git commit -m'Documentation update $(shell date)' && \
	git push origin gh-pages

docs-clean: ## Clear documentation build artifacts
	@rm -r docs/*


# Credit: https://marmelab.com/blog/2016/02/29/auto-documented-makefile.html
help: ## Print available recipes
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

