Source demos/_settings.tape

Output demos/quickstart.gif
Output demos/quickstart.mp4

Hide
Type `export PS1="$DEMO_PS1" && mkdir -p /tmp/.demo-home && echo 'PS1="$ "' > /tmp/.demo-home/.bash_profile && export HOME=/tmp/.demo-home`
Enter
Wait /\$/
Type `export PATH="$PWD/target/release:$PATH" && rm -rf "$HOME/.cx"`
Enter
Wait /\$/
Type "clear"
Enter
Wait /\$/
Show

Type "# cx: bootstrap conda in seconds"
Enter
Sleep 500ms

Type@80ms "cx bootstrap"
Enter
Wait /\$/
Sleep 2s

Type "# Create an environment with packages"
Enter
Sleep 500ms

Type@80ms "cx create -n demo python=3.12 numpy -y -q"
Enter
Wait /\$/
Sleep 2s

Type "# Activate with a subshell (no conda init needed)"
Enter
Sleep 500ms

Type@80ms "cx shell demo"
Enter
Wait /\$/
Sleep 1s

Type@80ms `python -c "import numpy; print(numpy.__version__)"`
Enter
Wait /\$/
Sleep 2s

Type "# Leave the environment"
Enter
Sleep 500ms

Type@80ms "exit"
Enter
Wait /\$/
Sleep 1s

Type "# That's it — single binary, no shell init, fast"
Enter
Sleep 3s
