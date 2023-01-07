#!/bin/bash -e

license_title="$(grep -F 'Copyright' LICENSE)"

for f in $(find . -name '*.rs' -not -path "*/target/*"); do
  if head -n 10 "$f" | grep -q -F "$license_title"; then
    continue
  fi

  {
    awk 'BEGIN {print "/*"}; END {print "*/"}; {indent="  "}; /^\s*$/ {indent=""}; { print indent $0 }' LICENSE
    echo
    cat "$f"
  } > "$f~"

  mv "$f~" "$f"
done

