Source demos/_settings.tape

Output demos/quickstart.gif

Source demos/_cx_setup.tape
Type `rm -rf "$CX_PREFIX"`
Enter
Wait /\$/
Type "clear"
Enter
Wait /\$/
Show

Type "# The first conda command bootstraps automatically"
Enter
Sleep 500ms

Type@80ms "cx info"
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

Type@80ms "cx spawn demo"
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
