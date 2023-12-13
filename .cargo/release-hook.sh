#!/usr/bin/env bash
set -e -u -o pipefail

echo "Preparing release $NEW_VERSION..."

#if [ $DRY_RUN = "false" ]; then
#  echo "Switching to branch 'main'..."
#  git checkout main
#  git pull origin main
#  echo "Merging 'develop'..."
#  git merge develop
#else
#  echo "Skipping branch gymnastics in DRY_RUN mode."
#fi
