Source demos/_settings.tape

Output demos/passthrough.gif

Source demos/_cx_setup.tape
Type `rm -rf "$CX_PREFIX" && cx create -n myenv python=3.12 pandas -y -q 2>/dev/null && cx remove -n myenv --all -y -q 2>/dev/null`
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
