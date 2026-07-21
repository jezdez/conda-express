Source demos/_settings.tape

Output demos/status.gif

Source demos/_cx_setup.tape
Type `rm -rf "$CX_PREFIX" && cx info >/dev/null 2>&1 && cx create -n science python=3.12 scipy -y -q 2>/dev/null`
Enter
Wait /\$/
Type "clear"
Enter
Wait /\$/
Show

Type "# cx info shows conda and prefix status"
Enter
Sleep 500ms

Type@80ms "cx info"
Enter
Wait /\$/
Sleep 3s

Type "# --version is conda's version"
Enter
Sleep 500ms

Type@80ms "cx --version"
Enter
Wait /\$/
Sleep 3s

Type "# List environments"
Enter
Sleep 500ms

Type@80ms "cx env list"
Enter
Wait /\$/
Sleep 3s
