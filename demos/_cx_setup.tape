Hide
Type `export PS1="$DEMO_PS1" && DEMO_HOME="$(mktemp -d /tmp/cx-demo-home.XXXXXX)" && echo 'PS1="$ "' > "$DEMO_HOME/.bash_profile" && export HOME="$DEMO_HOME"`
Enter
Wait /\$/
Type `export CX_BIN="${CX_BIN:-cx}" CXZ_BIN="${CXZ_BIN:-cxz}" CX_PREFIX="${CX_PREFIX:-$HOME/.conda/express}" && export PATH="$CX_PREFIX/condabin:$PATH"; cx() { "$CX_BIN" "$@"; }; cxz() { "$CXZ_BIN" "$@"; }`
Enter
Wait /\$/
