Source demos/_settings.tape

Output demos/quickstart.gif

Source demos/_cx_setup.tape
Type `cx bootstrap --force 2>/dev/null && cx create -n demo python=3.12 numpy -y -q 2>/dev/null && cx remove -n demo --all -y -q 2>/dev/null && rm -rf "$CX_HOME"`
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
