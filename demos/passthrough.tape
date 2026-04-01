Source demos/_settings.tape

Output demos/passthrough.gif
Output demos/passthrough.mp4

Hide
Type `export PS1="$DEMO_PS1" && mkdir -p /tmp/.demo-home && echo 'PS1="$ "' > /tmp/.demo-home/.bash_profile && export HOME=/tmp/.demo-home`
Enter
Wait /\$/
Type `export PATH="$PWD/target/release:$HOME/.cx/condabin:$PATH" && cx bootstrap --force 2>/dev/null`
Enter
Wait /\$/
Type "clear"
Enter
Wait /\$/
Show

Type "# cx delegates all conda commands transparently"
Enter
Sleep 500ms

Type@80ms "cx create -n myenv python=3.12 pandas -y -q"
Enter
Wait /\$/
Sleep 2s

Type@80ms "cx list -n myenv | head -15"
Enter
Wait /\$/
Sleep 2s

Type@80ms "cx search numpy --info 2>&1 | head -20"
Enter
Wait /\$/
Sleep 2s

Type@80ms "cx config --show channels"
Enter
Wait /\$/
Sleep 2s

Type "# Any conda command works — cx is just a thin wrapper"
Enter
Sleep 3s
