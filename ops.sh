
set -euo

#sed -i -E 's/\bdesc\b/body/g' $(find . -type f)
#sed -i -E 's/\bdescription\b/body/g' $(find . -type f)

sed -i -E 's/\bdesc\b/description/g' mise.toml
